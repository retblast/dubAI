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
use llm_connect::config::KoboldChatConfig;
use llm_connect::config::KoboldConfig;
use llm_connect::config::KoboldTTSConfig;
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
    input_audio: String,
    /// Output audio folder to store the dubbed files on
    #[arg(long, required = true)]
    output_folder: String,
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

#[derive(Debug)]
enum ParseURLError {
    CantReadPort,
    Unparseable,
}

// extract the host and address
fn parse_url_address(address: &str) -> Result<(String, u32), ParseURLError> {
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

fn setup_translator_cli(options: TranslatorCLI) -> TranslatorConfig {
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

fn setup_dubber_cli(options: DubberCLI) -> DubberConfig {
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
    let output_folder = {
        let mut temp_dir = options.output_folder;
        // Append "/" if necessary
        if temp_dir.chars().last() != Some('/') {
            temp_dir.push('/');
        }
        temp_dir
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

    dubber_config
}

// General setup
pub async fn setup_from_cli() {
    let cli = Cli::parse();

    //TODO: Maybe there's a better way to handle this?
    let mut dubai_config = DubAIConfig {
        ..Default::default()
    };

    match cli.mode {
        // Translator setup
        Mode::Translate(options) => {
            dubai_config.translator_config = setup_translator_cli(options);
            let srt_path = PathBuf::from(&dubai_config.translator_config.input_srt_path);
            let srt_file = open_input_file(&srt_path);
            let output_srt_path = PathBuf::from(&dubai_config.translator_config.output_srt_path);
            let output_srt_file = open_output_file(&output_srt_path);
            let srt_fragments = match get_srt_fragments(&srt_file) {
                Ok(fragments) => fragments,
                Err(why) => {
                    //TODO: Implement display for the error types someday
                    println!("{:?}", why);
                    std::process::exit(1);
                }
            };
            let (host, port) = parse_url_address(&dubai_config.translator_config.llm_address)
                .expect("Failed to parse the URL");
            let kobold_chat_config = KoboldChatConfig::new(&dubai_config.translator_config.model);
            let kobold_config = KoboldConfig::new(&host, &port, None, Some(kobold_chat_config));
            koboldcpp_start(&kobold_config).await;
            translate_loaded_srt_fragments(
                &srt_fragments,
                &dubai_config.translator_config,
                &output_srt_file,
            )
            .await;
        }
        Mode::Dub(options) => {
            dubai_config.dubber_config = setup_dubber_cli(options);
            let srt_path = PathBuf::from(&dubai_config.dubber_config.input_srt);
            let srt_file = open_input_file(&srt_path);
            let srt_fragments = match get_srt_fragments(&srt_file) {
                Ok(fragments) => fragments,
                Err(why) => {
                    //TODO: Implement display for the error types someday
                    println!("{:?}", why);
                    std::process::exit(1);
                }
            };
            let voice_refs = create_voice_references(
                &srt_fragments,
                &dubai_config.dubber_config.input_audio,
                &dubai_config.dubber_config.voice_refs_dir,
            );
            let (host, port) = parse_url_address(&dubai_config.dubber_config.llm_address)
                .expect("Failed to parse the URL");
            let kobold_tts_config = KoboldTTSConfig::new(
                &dubai_config.dubber_config.model,
                &dubai_config.dubber_config.wavtokenizer,
                &dubai_config.dubber_config.voice_refs_dir,
            );
            // TEST ONLY
            // println!("llm_address: {}", &dubai_config.dubber_config.llm_address);
            // println!(
            //     "llm_address: {:?}",
            //     parse_url_address(&dubai_config.dubber_config.llm_address)
            // );
            let kobold_config = KoboldConfig::new(&host, &port, Some(kobold_tts_config), None);
            println!("Voice references: {:#?}", voice_refs);
            koboldcpp_start(&kobold_config).await;
            dub_srt_file(&srt_fragments, &dubai_config.dubber_config, voice_refs).await;
        }
    }
}
