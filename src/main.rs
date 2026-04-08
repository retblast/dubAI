use std::path::Path;
use std::path::PathBuf;

use std::str::FromStr;

use crate::config::DubConfig;
use crate::config::set_translator_config;
use crate::file_ops::open_input_file;
use crate::file_ops::open_output_file;
use crate::srt_ops::process_srt_file;
use clap::Parser;
use llm_connect::connection::koboldcpp_start;
use llm_connect::connection::openai_tts_send_prompt;

mod config;
mod file_ops;
mod srt_ops;
#[derive(Parser)]
#[command(name = "dubai")]
#[command(version, about = "AI dubbing toolbox", long_about = "To dub things.")]
struct Cli {
    /// Language to dub from (fed to the AI)
    #[arg(default_value = "English", short = 'l', long)]
    input_language: Option<String>,

    /// Language to dub to (fed to the AI)
    #[arg(default_value = "English", short = 'L', long)]
    output_language: Option<String>,

    /// URL address of the LLM
    #[arg(long)]
    address: Option<String>,

    /// Extra context for the dubbing LLMs
    #[arg(long)]
    extra_context: Option<String>,

    /// Input SRT file to translate
    #[arg(long)]
    input_srt_file: Option<String>,

    /// Output SRT file to translate
    #[arg(long)]
    output_srt_file: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let input_language = match cli.input_language {
        Some(input_language) => input_language,
        None => panic!("No language to dub from specified."),
    };
    let output_language = match cli.output_language {
        Some(output_language) => output_language,
        None => panic!("No language to dub to specified."),
    };
    let llm_address = match cli.address {
        Some(address) => address,
        None => panic!("No URL address for the LLM connection has been specified."),
    };
    let extra_context = match cli.extra_context {
        Some(extra_context) => extra_context,
        None => {
            println!("No extra context fed to the dubbing LLMs");
            "".to_string()
        }
    };

    let input_srt_path = match cli.input_srt_file {
        Some(input_srt_path) => PathBuf::from(input_srt_path),
        None => panic!("No input SRT file provided."),
    };

    let output_srt_path = match cli.output_srt_file {
        Some(output_srt_path) => PathBuf::from(output_srt_path),
        None => {
            println!(
                "No outpuf file specified. \".srt\" will be appended to the input file to form an output file"
            );
            PathBuf::from(Path::new(input_srt_path.as_path()).with_added_extension("srt"))
        }
    };
    let mut dub_config: DubConfig = { Default::default() };

    set_translator_config(
        &mut dub_config,
        llm_address,
        input_language,
        output_language,
        extra_context,
    );

    let input_srt_file = open_input_file(&input_srt_path);
    let output_srt_file = open_output_file(&output_srt_path);
    process_srt_file(input_srt_file, output_srt_file, &dub_config).await;
}
