use crate::config::DubAIConfig;
use crate::config::DubberConfig;
use crate::config::TranslatorConfig;
use crate::dub::create_voice_references;
use crate::dub::dub_srt_file;
use crate::file_ops::open_input_file;
use crate::file_ops::open_output_file;
use crate::srt_ops::get_srt_fragments;
use crate::translate::translate_loaded_srt_fragments;
use clap::Parser;
use clap::Subcommand;
use llm_connect::connection::koboldcpp_start;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct TranslatorCLI {
    /// Language to translate from (fed to the AI)
    #[arg(default_value = "English", short = 'l', long, required = true)]
    input_language: String,

    /// Model to use for translation
    #[arg(short, long, required = true)]
    model: String,

    /// Max tokens for the translator
    #[arg(short, long)]
    // u8 ought to be enough for a line, I reckon?
    max_tokens: Option<u8>,

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
    input_srt_file: String,

    /// Output SRT file to translate
    #[arg(long)]
    output_srt_file: Option<String>,
}

#[derive(Parser)]
struct DubberCLI {
    /// URL address of the LLM
    #[arg(long)]
    address: Option<String>,
    /// Model to use for dubbing
    #[arg(short, long, required = true)]
    model: String,
    /// Wavtokenizer to use for dubbing
    #[arg(short, long, required = true)]
    wavtokenizer: String,
    /// Input audio file to use for dubbing
    #[arg(long, required = true)]
    input_audio: Option<String>,
    /// Output audio folder to store the dubbed files on
    #[arg(long)]
    output_folder: Option<String>,
    /// Input srt file to use for dubbing
    #[arg(long, required = true)]
    input_srt: String,
    /// Directory where the voice references are
    #[arg(long, required = true)]
    voice_refs_dir: String,
    /// Language to dub to (fed to the AI)
    // kind of a dead feature lol
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

// extract the host and address
fn parse_url_address(address: &String) -> (String, u32) {
    let (host, port): (String, u32) = match address.rsplit_once(':') {
        Some((host_string, address_string)) => (
            host_string.to_string(),
            match address_string.parse::<u32>() {
                Ok(port) => port,
                Err(why) => {
                    panic!("Failed to get port from the URI: {why}")
                }
            },
        ),
        None => {
            panic!("Failed to parse URI.")
        }
    };
    return (host, port);
}

fn setup_translator_cli(options: TranslatorCLI, dubai_config: &mut DubAIConfig) {
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

    dubai_config.translator_config = TranslatorConfig::new(
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
}

fn setup_dubber_cli(options: DubberCLI, dubai_config: &mut DubAIConfig) {
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
    let input_audio = match options.input_audio {
        Some(audio) => audio,
        None => panic!("No input audio for the dubber LLM has been specified."),
    };
    let output_folder = match options.output_folder {
        Some(mut dir) => {
            // Append "/" if necessary
            if dir.chars().last() != Some('/') {
                dir.push('/');
            }
            dir
        }
        None => panic!("No output folder for the dubber LLM has been specified."),
    };
    let input_srt = options.input_srt;
    let voice_refs_dir = {
        let mut temp_dir = options.voice_refs_dir;
        // Append "/" if necessary
        if temp_dir.chars().last() != Some('/') {
            temp_dir.push('/');
        }
        temp_dir
    };
    let output_language = match options.output_language {
        Some(address) => address,
        None => panic!("No language to dub to specified."),
    };

    dubai_config.dubber_config = DubberConfig::new(
        llm_address,
        model,
        wavtokenizer,
        input_audio,
        input_srt,
        voice_refs_dir,
        output_language,
        output_folder,
    );
}

// General setup
pub async fn setup_from_cli(dubai_config: &mut DubAIConfig) {
    let cli = Cli::parse();
    match cli.mode {
        // Translator setup
        Mode::Translate(options) => {
            setup_translator_cli(options, dubai_config);
            let srt_path = PathBuf::from(&dubai_config.translator_config.input_srt_path);
            let srt_file = open_input_file(&srt_path);
            let output_srt_path = PathBuf::from(&dubai_config.translator_config.output_srt_path);
            let output_srt_file = open_output_file(&output_srt_path);
            let srt_fragments = get_srt_fragments(&srt_file);
            let (host, port) = parse_url_address(&dubai_config.translator_config.llm_address);
            koboldcpp_start(
                &"chat".to_string(),
                &host,
                &port,
                &dubai_config.translator_config.model,
                &"".to_owned(),
                &dubai_config.dubber_config.voice_refs_dir,
            )
            .await;
            translate_loaded_srt_fragments(
                &srt_fragments,
                &dubai_config.translator_config,
                &output_srt_file,
            )
            .await;
        }
        Mode::Dub(options) => {
            setup_dubber_cli(options, dubai_config);
            let srt_path = PathBuf::from(&dubai_config.dubber_config.input_srt);
            let srt_file = open_input_file(&srt_path);
            let srt_fragments = get_srt_fragments(&srt_file);
            let voice_refs = create_voice_references(
                &srt_fragments,
                &dubai_config.dubber_config.input_audio,
                &dubai_config.dubber_config.voice_refs_dir,
            );
            let (host, port) = parse_url_address(&dubai_config.translator_config.llm_address);
            println!("Voice references: {:#?}", voice_refs);
            koboldcpp_start(
                &"tts".to_string(),
                &host,
                &port,
                &dubai_config.dubber_config.model,
                &dubai_config.dubber_config.wavtokenizer,
                &dubai_config.dubber_config.voice_refs_dir,
            )
            .await;
            dub_srt_file(&srt_fragments, &dubai_config.dubber_config, voice_refs).await;
        }
    }
}
