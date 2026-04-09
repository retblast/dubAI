use crate::cli::setup_from_cli;
use crate::config::DubConfig;
use crate::config::set_translator_config;
use crate::file_ops::open_input_file;
use crate::file_ops::open_output_file;
use crate::srt_ops::process_srt_file;
use llm_connect::connection::koboldcpp_start;

mod cli;
mod config;
mod file_ops;
mod srt_ops;

#[tokio::main]
async fn main() {
    let mut dub_config: DubConfig = { Default::default() };
    setup_from_cli(&mut dub_config);
    let input_srt_file = open_input_file(&dub_config.translator_config.input_srt_path);
    let output_srt_file = open_output_file(&dub_config.translator_config.output_srt_path);
    process_srt_file(input_srt_file, output_srt_file, &dub_config).await;
}
