use crate::config::DubConfig;
use llm_connect::connection::openai_chat_send_prompt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Default, Clone, Debug)]
pub struct SRTFragment {
    pub index: usize,
    pub timing: String,
    pub line: String,
}

pub fn get_srt_timings(srt_fragment: &SRTFragment) -> (String, String) {
    let (start, end) = match srt_fragment.timing.split_once("-->") {
        Some((start, end)) => (start.trim().replace(',', "."), end.trim().replace(',', ".")),
        None => panic!(
            "Failed to read SRT timings at SRT index: {}",
            srt_fragment.index
        ),
    };
    return (start, end);
}

// Reads from a buffer, returns a SRTFragment and the buffer, for the next iteration
// or smth else
pub fn get_srt_fragments(srt_file: &File) -> Vec<SRTFragment> {
    let mut vector_fragments = Vec::new();
    let mut buffered_srt_file = BufReader::new(srt_file);
    let mut finished_reading = false;
    let mut current_line = String::new();
    let mut current_index = 0;
    let mut current_timing = String::new();
    let mut finished_reading = false;

    let mut current_fragment = SRTFragment {
        ..Default::default()
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
                return vector_fragments;
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
                vector_fragments.push(current_fragment);
                // Clean up for next iteration
                current_index = 0;
                current_timing = "".to_owned();
            }
        }
        current_index = match current_line.trim().parse::<usize>() {
            // Is this the correct way to "do nothing"?
            Err(_) => current_index,
            Ok(number) => number,
        };
    }
    return vector_fragments;
}

// TODO: Fix
// pub async fn process_srt_file(
//     input_srt_file: File,
//     output_srt_file: File,
//     dub_config: &DubConfig,
// ) -> () {
//     let mut buffered_reader = BufReader::new(input_srt_file);
//     let mut current_srt_fragment = Vec::new();
//     let mut translated_fragment: SRTFragment;
//     let mut finished_reading = false;

//     (current_srt_fragment, finished_reading) = get_srt_fragments(&input_srt_file);

//     // Finished reading = finished translating
//     if finished_reading {
//         return;
//     }

//     translated_fragment = current_srt_fragment.clone();
//     println!("Translating: {}", &current_srt_fragment.line);
//     // TODO: implement translate_line
//     translated_fragment.line = match translate_line(&current_srt_fragment.line, &dub_config).await {
//         Ok(string) => string,
//         Err(why) => {
//             panic!("Failed to translate the current line, {}", why);
//         }
//     };
//     write_srt_file(&translated_fragment, output_srt_file);
// }

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
