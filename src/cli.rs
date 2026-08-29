use crate::config::DubberConfig;
use crate::config::TranslatorConfig;
use crate::dub::DubError;
use clap::Parser;
use clap::Subcommand;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
pub struct TranslatorCLI {
    /// Language to translate from (fed to the AI)
    #[arg(default_value = "English", short = 'l', long, required = true)]
    input_language: String,

    /// Model to use for translation
    #[arg(short, long, required = true)]
    model: PathBuf,

    /// Max tokens for the translator
    #[arg(short, long)]
    // u16 ought to be enough for a line, I reckon?
    max_tokens: Option<u16>,

    /// Temperature for the translator
    #[arg(short, long)]
    temperature: Option<f32>,

    /// Language to translate to (fed to the AI)
    #[arg(default_value = "English", short = 'L', long, required = true)]
    output_language: String,

    /// URL address of the LLM
    #[arg(long)]
    address: Option<String>,

    /// Extra context for the translation LLMs
    #[arg(long)]
    extra_context: Option<String>,

    /// Input SRT file to translate
    #[arg(long, required = true)]
    input_srt_file: PathBuf,

    /// Output SRT file to translate
    #[arg(long)]
    output_srt_file: Option<PathBuf>,
}

#[derive(Parser)]
pub struct DubberCLI {
    /// URL address of the LLM
    #[arg(long)]
    address: Option<String>,
    /// Model to use for dubbing
    #[arg(short, long, required = true)]
    model: PathBuf,
    /// Wavtokenizer to use for dubbing
    #[arg(short, long, required = true)]
    wavtokenizer: PathBuf,
    /// Input audio file to use for dubbing
    #[arg(long, required = true)]
    input_audio: PathBuf,
    /// Output audio folder to store the dubbed files on
    #[arg(long, required = true)]
    output_folder: PathBuf,
    /// Input srt file to use for dubbing
    #[arg(long, required = true)]
    input_srt: PathBuf,
    /// Directory where the voice references are
    #[arg(long, required = true)]
    voice_refs_dir: PathBuf,
    /// Language to dub to (fed to the AI)
    // kind of a dead feature lol
    #[arg(default_value = "English", short = 'L', long)]
    output_language: Option<String>,
}

#[derive(Subcommand)]
pub enum Mode {
    /// Translation (SRT files) mode
    Translate(TranslatorCLI),
    /// Dubbing mode
    Dub(DubberCLI),
}

#[derive(Parser)]
#[command(name = "dubai")]
#[command(version, about = "AI dubbing toolbox", long_about = "To dub things.")]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Debug)]
pub enum ParseURLError {
    CantReadPort,
    Unparseable,
}

// extract the host and address
pub fn parse_url_address(address: &str) -> Result<(String, u32), ParseURLError> {
    let (host, port): (String, u32) = match address.rsplit_once(':') {
        Some((host_string, address_string)) => (
            host_string.to_string(),
            match address_string.parse::<u32>() {
                Ok(port) => port,
                // I throw the 'why' here
                Err(_) => return Err(ParseURLError::CantReadPort),
            },
        ),
        None => return Err(ParseURLError::Unparseable),
    };
    return Ok((host, port));
}

pub fn setup_translator_cli(options: TranslatorCLI) -> TranslatorConfig {
    let input_language;
    let output_language;
    let llm_address;
    let model;
    let max_tokens;
    let temperature;
    let extra_context;
    let input_srt_path;
    let output_srt_path;

    model = options.model;
    max_tokens = match options.max_tokens {
        Some(max_tokens) => max_tokens,
        None => {
            println!("No maximum number of tokens for translation specified. Using 255.");
            255
        }
    };
    temperature = match options.temperature {
        Some(temperature) => temperature,
        None => {
            println!("No temperature for translation specified. Using 0.6.");
            0.6
        }
    };
    input_language = options.input_language;
    output_language = options.output_language;
    llm_address = match options.address {
        Some(address) => address,
        None => {
            println!("No URL address for the translator LLM connection has been specified.");
            println!("Assuming: http://localhost:5001");
            "http://localhost:5001".to_string()
        }
    };
    extra_context = match options.extra_context {
        Some(extra_context) => extra_context,
        None => {
            println!("No extra context fed to the translation LLM");
            "".to_string()
        }
    };
    input_srt_path = PathBuf::from(options.input_srt_file);
    output_srt_path = match options.output_srt_file {
        Some(output_srt_path) => PathBuf::from(output_srt_path),
        None => {
            println!(
                "No outpuf file specified. \".srt\" will be appended to the input file to form an output file"
            );
            PathBuf::from(Path::new(input_srt_path.as_path()).with_added_extension("srt"))
        }
    };

    let translator_config = TranslatorConfig::new(
        llm_address,
        model,
        temperature,
        max_tokens,
        input_language,
        output_language,
        extra_context,
        input_srt_path,
        output_srt_path,
    );

    translator_config
}

pub fn setup_dubber_cli(options: DubberCLI) -> Result<DubberConfig, DubError> {
    let llm_address = match options.address {
        Some(address) => address,
        None => {
            println!("No URL address for the translator LLM connection has been specified.");
            println!("Assuming: http://localhost:5001");
            "http://localhost:5001".to_string()
        }
    };
    let model = options.model;
    let wavtokenizer = options.wavtokenizer;
    let input_audio = options.input_audio;

    let output_folder = match options.output_folder.is_dir() {
        true => options.output_folder,
        false => return Err(DubError::FolderRequired),
    };

    let input_srt = options.input_srt;
    let voice_refs_dir = match options.voice_refs_dir.is_dir() {
        true => options.voice_refs_dir,
        false => return Err(DubError::FolderRequired),
    };

    let output_language = match options.output_language {
        Some(address) => address,
        None => panic!("No language to dub to specified."),
    };

    let dubber_config = DubberConfig::new(
        llm_address,
        model,
        wavtokenizer,
        input_audio,
        input_srt,
        voice_refs_dir,
        output_language,
        output_folder,
    );

    Ok(dubber_config)
}

// General setup
// pub async fn setup_from_cli() -> Result<(), dubaiError> {

// }
