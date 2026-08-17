use std::path::PathBuf;

#[derive(Default)]
pub struct DubAIConfig {
    // TODO: Set transcriber config
    // put transcriber_config: TranscriberConfig,
    pub translator_config: TranslatorConfig,
    pub dubber_config: DubberConfig,
}

#[derive(Default)]
pub struct DubberConfig {
    pub llm_address: String,
    pub model: String,
    pub wavtokenizer: String,
    pub input_audio: String,
    pub input_srt: String,
    pub voice_refs_dir: String,
    // Uneeded, but keep just in case
    // pub input_language: String,
    // Not yet supported by koboldCPP
    pub output_language: String,
    pub output_folder: String,
}

impl DubberConfig {
    pub fn new(
        llm_address: String,
        model: String,
        wavtokenizer: String,
        input_audio: String,
        input_srt: String,
        voice_refs_dir: String,
        output_language: String,
        output_folder: String,
    ) -> Self {
        Self {
            llm_address,
            model,
            wavtokenizer,
            input_audio,
            input_srt,
            voice_refs_dir,
            output_language,
            output_folder,
        }
    }
}

#[derive(Default)]
pub struct TranslatorConfig {
    pub llm_address: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u16,
    pub input_language: String,
    pub output_language: String,
    pub extra_context: String,
    pub input_srt_path: PathBuf,
    pub output_srt_path: PathBuf,
}

impl TranslatorConfig {
    pub fn new(
        llm_address: String,
        model: String,
        temperature: f32,
        max_tokens: u16,
        input_language: String,
        output_language: String,
        extra_context: String,
        input_srt_path: PathBuf,
        output_srt_path: PathBuf,
    ) -> Self {
        Self {
            llm_address,
            model,
            temperature,
            max_tokens,
            input_language,
            output_language,
            extra_context,
            input_srt_path,
            output_srt_path,
        }
    }
}
