use crate::config::TranslatorConfig;
use crate::file_ops::write_srt_file;
use crate::srt_ops::SRTFragment;
use llm_connect::connection::openai_chat_send_prompt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

// Translates the loaded SRT fragments
// and saves them in the output file
pub async fn translate_loaded_srt_fragments(
    srt_fragments: &Vec<SRTFragment>,
    translator_config: &TranslatorConfig,
    output_file: &File,
) -> () {
    for current_srt_fragment in srt_fragments {
        let mut translated_fragment = current_srt_fragment.clone();
        println!("Translating: {}", &current_srt_fragment.line);
        // TODO: implement translate_line
        translated_fragment.line =
            match translate_line(&current_srt_fragment.line, &translator_config).await {
                Ok(string) => string,
                Err(why) => {
                    panic!("Failed to translate the current line, {}", why);
                }
            };
        write_srt_file(&translated_fragment, output_file);
    }
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
    translator_config: &TranslatorConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    // Because we can't access fields in format!
    let input_language = &translator_config.input_language;
    let output_language = &translator_config.output_language;
    let extra_context = &translator_config.extra_context;

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
        &translator_config.llm_address,
        &system_prompt,
        &user_prompt,
        &translator_config.temperature,
        &(translator_config.max_tokens as u32),
        5,
    )
    .await?;
    return Ok(response.choices[0].message.content.trim().to_string());
}
