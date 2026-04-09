use crate::config::DubConfig;
use crate::config::set_dubber_config;
use crate::config::set_translator_config;
use clap::Parser;
use clap::Subcommand;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct TranslatorCLI {
    /// Language to trandlate from (fed to the AI)
    #[arg(default_value = "English", short = 'l', long)]
    input_language: Option<String>,

    /// Language to translate to (fed to the AI)
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

#[derive(Parser)]
struct DubberCLI {
    /// URL address of the LLM
    #[arg(long)]
    address: Option<String>,
    /// Language to dub to (fed to the AI)
    #[arg(default_value = "English", short = 'L', long)]
    output_language: Option<String>,
}

#[derive(Subcommand)]
enum Mode {
    /// Translation (SRT files) mode
    Translate(TranslatorCLI),
    /// Dubbing mode
    Dub(DubberCLI),
}

#[derive(Parser)]
#[command(name = "dubai")]
#[command(version, about = "AI dubbing toolbox", long_about = "To dub things.")]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

fn setup_translator_cli(options: TranslatorCLI, dub_config: &mut DubConfig) {
    let input_language;
    let output_language;
    let llm_address;
    let extra_context;
    let input_srt_path;
    let output_srt_path;

    input_language = match options.input_language {
        Some(input_language) => input_language,
        None => panic!("No language to dub from specified."),
    };
    output_language = match options.output_language {
        Some(output_language) => output_language,
        None => panic!("No language to dub to specified."),
    };
    llm_address = match options.address {
        Some(address) => address,
        None => {
            panic!("No URL address for the translator LLM connection has been specified.")
        }
    };
    extra_context = match options.extra_context {
        Some(extra_context) => extra_context,
        None => {
            println!("No extra context fed to the translation LLM");
            "".to_string()
        }
    };
    input_srt_path = match options.input_srt_file {
        Some(input_srt_path) => PathBuf::from(input_srt_path),
        None => panic!("No input SRT file provided."),
    };
    output_srt_path = match options.output_srt_file {
        Some(output_srt_path) => PathBuf::from(output_srt_path),
        None => {
            println!(
                "No outpuf file specified. \".srt\" will be appended to the input file to form an output file"
            );
            PathBuf::from(Path::new(input_srt_path.as_path()).with_added_extension("srt"))
        }
    };
    set_translator_config(
        dub_config,
        llm_address,
        input_language,
        output_language,
        extra_context,
        input_srt_path,
        output_srt_path,
    );
}

fn setup_dubber_cli(options: DubberCLI, dub_config: &mut DubConfig) {
    let llm_address = match options.address {
        Some(address) => address,
        None => panic!("No URL address for the dubber LLM connection has been specified."),
    };
    let output_language = match options.output_language {
        Some(address) => address,
        None => panic!("No language to dub to specified."),
    };
    set_dubber_config(dub_config, llm_address, output_language);
}

pub fn setup_from_cli(dub_config: &mut DubConfig) {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Translate(options) => setup_translator_cli(options, dub_config),
        Mode::Dub(options) => setup_dubber_cli(options, dub_config),
    }
}
