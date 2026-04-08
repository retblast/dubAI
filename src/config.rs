#[derive(Default)]
pub struct DubConfig {
    pub translator_config: TranslatorConfig,
}

#[derive(Default)]
pub struct TranslatorConfig {
    pub llm_address: String,
    pub input_language: String,
    pub output_language: String,
    pub extra_context: String,
}

pub fn set_translator_config(
    dub_config: &mut DubConfig,
    llm_address: String,
    input_language: String,
    output_language: String,
    extra_context: String,
) {
    let translator_config = TranslatorConfig {
        llm_address: llm_address,
        input_language: input_language,
        output_language: output_language,
        extra_context: extra_context,
    };
    dub_config.translator_config = translator_config;
}
