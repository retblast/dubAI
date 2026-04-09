use crate::config::DubConfig;
use crate::config::DubberConfig;
use crate::config::TranslatorConfig;
use crate::file_ops::write_srt_file;
use llm_connect::connection::openai_chat_send_prompt;
use llm_connect::connection::openai_tts_send_prompt;
use std::clone;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone, Debug)]
pub struct SRTFragment {
    pub index: u32,
    pub timing: String,
    pub line: String,
}

// Reads from a buffer, returns a SRTFragment and the buffer, for the next iteration
// or smth else
pub fn get_srt_fragment(buffered_srt_file: &mut BufReader<File>) -> (SRTFragment, bool) {
    let mut finished_reading = false;
    let mut current_line = String::new();
    let mut current_index = 0;
    let mut current_timing = String::new();
    let mut finished_reading = false;

    let mut current_fragment = SRTFragment {
        index: 0,
        timing: String::new(),
        line: String::new(),
    };
    while !finished_reading {
        // clear line
        current_line.clear();
        // get a new line
        match buffered_srt_file.read_line(&mut current_line) {
            Err(why) => println!("Couldn't read: {}", why),
            Ok(1_usize..) => {}
            // Finished reading the fragment
            Ok(0_usize) => {
                finished_reading = true;
                continue;
            }
        }

        // After reading the current index
        if current_index != 0 {
            // Read the timing first
            if current_timing.is_empty() {
                current_timing = current_line.clone().trim().to_owned();
            } else {
                // Finally, we also now have the current line, so
                // assemble the whole fragment
                current_fragment = SRTFragment {
                    index: current_index,
                    timing: current_timing,
                    line: current_line.clone().trim().to_owned(),
                };
                // Assemble current fragment for potential future use
                current_index = 0;
                current_timing = "".to_owned();
            }
        }

        current_index = match current_line.trim().parse::<u32>() {
            // Is this the correct way to "do nothing"?
            Err(_) => current_index,
            Ok(number) => number,
        };
    }

    return (current_fragment, finished_reading);
}

pub async fn process_srt_file(
    input_srt_file: File,
    output_srt_file: File,
    dub_config: &DubConfig,
) -> () {
    let mut buffered_reader = BufReader::new(input_srt_file);
    let mut current_srt_fragment: SRTFragment;
    let mut translated_fragment: SRTFragment;
    let mut finished_reading = false;

    (current_srt_fragment, finished_reading) = get_srt_fragment(&mut buffered_reader);

    // Finished reading = finished translating
    if finished_reading {
        return;
    }

    translated_fragment = current_srt_fragment.clone();
    println!("Translating: {}", &current_srt_fragment.line);
    // TODO: implement translate_line
    translated_fragment.line = match translate_line(&current_srt_fragment.line, &dub_config).await {
        Ok(string) => string,
        Err(why) => {
            panic!("Failed to translate the current line, {}", why);
        }
    };
    write_srt_file(&translated_fragment, output_srt_file);
}

// checks the progress of the translated srt
// I assume that it is well formed... so
// TODO: validation/checking of the output srt.
pub fn get_translated_srt_progress(read_buffer: &mut BufReader<&File>) -> u16 {
    let mut progress_index: u16 = 0;
    let mut finished_reading = false;
    let mut current_line = String::new();
    // should loop until EOF
    while !finished_reading {
        // clear buffer
        current_line.clear();
        match read_buffer.read_line(&mut current_line) {
            Err(why) => println!("Couldn't read: {}", why),
            Ok(1_usize..) => {}
            Ok(0) => {
                finished_reading = true;
                continue;
            }
        }

        // We only care about the index number
        progress_index = match current_line.parse::<u16>() {
            // Is this the correct way to "do nothing"?
            Err(_) => progress_index,
            Ok(number) => number,
        };
    }
    return progress_index;
}

// Translates a given line
pub async fn translate_line(
    line: &String,
    dub_config: &DubConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    // Because we can't access fields in format!
    let input_language = &dub_config.translator_config.input_language;
    let output_language = &dub_config.translator_config.output_language;
    let extra_context = &dub_config.translator_config.extra_context;

    let system_prompt = format!(
        "
        You are an AI translator, translating lines from a an SRT file in {input_language} to {output_language}.
        Your response will include only the translated text, only in the requested language. Nothing more.
        Only quote stuff when characters (or something) explain a thing, use single quotes,
        like this: \" Translated text 'example'\"
        Only give 1 translation answer.
        Follow this to the letter.
        Follow this extra directives too: {extra_context}.
        "
    );
    let user_prompt = format!("This is the line that you have to translate: {line}");
    let response = openai_chat_send_prompt(
        &dub_config.translator_config.llm_address,
        &system_prompt,
        &user_prompt,
        &0.6,
        &300,
    )
    .await?;
    return Ok(response.choices[0].message.content.trim().to_string());
}

// Dubs a line
// Creates a index_dubbed.mp3 file
pub async fn dub_line(
    dubber_config: &DubberConfig,
    output_folder: &String,
    line_to_dub: &String,
    voice_ref: &String,
) {
    // The output filename is: output_folder + the index of the voice ref + _dubbed.mp3
    let output_filename = {
        let mut cloned_voice_ref = voice_ref.clone();
        let underscore_idx = match cloned_voice_ref.find("_") {
            Some(number) => number,
            None => panic!(
                "dub_line failed to find the underscore, index: {}",
                voice_ref
            ),
        };
        // Here I
        cloned_voice_ref.replace_range(underscore_idx + 1.., "dubbed.mp3");
        cloned_voice_ref.insert_str(0, output_folder);
        cloned_voice_ref
    };
    match openai_tts_send_prompt(
        &dubber_config.llm_address,
        &output_filename,
        &"kcpp".to_string(),
        line_to_dub,
        voice_ref,
    )
    .await
    {
        Ok(_) => {
            let current_dir = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(_) => panic!("Couldn't get current directory"),
            };
            let dubbing_path = Path::new(current_dir.as_path()).join(&output_filename);
            match fs::rename(&output_filename, dubbing_path).await {
                Ok(_) => {}
                Err(why) => println!(
                    "Couldn't put the generated audio file in its folder, because {}",
                    why
                ),
            };
        }
        Err(why) => println!(
            "Failed to generate: {}, because of: {}",
            output_filename, why
        ),
    };
}

// Dub an SRT file
// Requires a running LLM
pub fn dub_srt_file(
    srt_file: File,
    dubber_config: &DubberConfig,
    line_to_dub: &String,
    voice_ref: &String,
) {
    let mut buffered_reader = BufReader::new(srt_file);
    let mut current_srt_fragment: SRTFragment;
    let mut finished_reading = false;
    while !finished_reading {
        (current_srt_fragment, finished_reading) = get_srt_fragment(&mut buffered_reader);
    }
}
