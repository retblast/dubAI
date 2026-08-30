use crate::config::TranslatorConfig;
use crate::srt_ops::SRTFragment;
use llm_connect::connection::LlmConnectionError;
use llm_connect::connection::openai_chat_send_prompt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Debug)]
pub enum TranslateError {
    LlmError(LlmConnectionError),
}

impl From<LlmConnectionError> for TranslateError {
    fn from(error: LlmConnectionError) -> Self {
        Self::LlmError(error)
    }
}

impl Display for TranslateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LlmError(error) => write!(f, "Error with the LLM: {error}"),
        }
    }
}

// Translates the loaded SRT fragments
pub async fn translate_loaded_srt_fragments(
    srt_fragments: &[SRTFragment],
    translator_config: &TranslatorConfig,
) -> Result<Vec<SRTFragment>, TranslateError> {
    let mut output_fragments: Vec<SRTFragment> = Vec::new();
    for current_srt_fragment in srt_fragments {
        let mut translated_fragment = current_srt_fragment.clone();
        println!(
            "Translating: {}",
            &current_srt_fragment.get_flattened_lines()
        );
        // we can't avoid saving the translated line as a singular line
        translated_fragment.subtitle_lines.push(
            translate_line(
                &current_srt_fragment.get_flattened_lines(),
                &translator_config,
            )
            .await?,
        );
        output_fragments.push(translated_fragment);
        // write_srt_file(&translated_fragment, output_file);
    }
    Ok(output_fragments)
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
) -> Result<String, TranslateError> {
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
