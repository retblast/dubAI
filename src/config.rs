use std::path::PathBuf;

#[derive(Default)]
pub struct DubConfig {
    pub translator_config: TranslatorConfig,
    pub dubber_config: DubberConfig,
}

#[derive(Default)]
pub struct DubberConfig {
    pub llm_address: String,
    // Uneeded, but keep just in case
    // pub input_language: String,
    // Not yet supported by koboldCPP
    pub output_language: String,
}

#[derive(Default)]
pub struct TranslatorConfig {
    pub llm_address: String,
    pub input_language: String,
    pub output_language: String,
    pub extra_context: String,
    pub input_srt_path: PathBuf,
    pub output_srt_path: PathBuf,
}

pub fn set_translator_config(
    dub_config: &mut DubConfig,
    llm_address: String,
    input_language: String,
    output_language: String,
    extra_context: String,
    input_srt_path: PathBuf,
    output_srt_path: PathBuf,
) {
    let translator_config = TranslatorConfig {
        llm_address: llm_address,
        input_language: input_language,
        output_language: output_language,
        extra_context: extra_context,
        input_srt_path: input_srt_path,
        output_srt_path: output_srt_path,
    };
    dub_config.translator_config = translator_config;
}

pub fn set_dubber_config(dub_config: &mut DubConfig, llm_address: String, output_language: String) {
    let dubber_config = DubberConfig {
        llm_address: llm_address,
        output_language: output_language,
    };
    dub_config.dubber_config = dubber_config;
}
