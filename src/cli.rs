use crate::config::DubConfig;
use crate::config::set_dubber_config;
use crate::config::set_translator_config;
use crate::dub::create_voice_references;
use crate::dub::dub_srt_file;
use crate::file_ops::open_input_file;
use crate::file_ops::open_output_file;
use crate::srt_ops::get_srt_fragments;
use crate::srt_ops::translate_srt_file;
use clap::Parser;
use clap::Subcommand;
use llm_connect::connection::koboldcpp_start;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct TranslatorCLI {
    /// Language to trandlate from (fed to the AI)
    #[arg(default_value = "English", short = 'l', long)]
    input_language: Option<String>,

    /// Model to use for translation
    #[arg(short, long)]
    model: Option<String>,

    /// Model to use for translation
    #[arg(short, long)]
    // u8 ought to be enough for a line, I reckon?
    max_tokens: Option<u8>,

    /// Model to use for translation
    #[arg(short, long)]
    temperature: Option<f32>,

    /// Language to translate to (fed to the AI)
    #[arg(default_value = "English", short = 'L', long)]
    output_language: Option<String>,

    /// URL address of the LLM
    #[arg(long)]
    address: Option<String>,

    /// Extra context for the translation LLMs
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
    /// Model to use for dubbing
    #[arg(short, long)]
    model: Option<String>,
    /// Wavtokenizer to use for dubbing
    #[arg(short, long)]
    wavtokenizer: Option<String>,
    /// Input audio file to use for dubbing
    #[arg(long)]
    input_audio: Option<String>,
    /// Output audio folder to store the dubbed files on
    #[arg(long)]
    output_folder: Option<String>,
    /// Input srt file to use for dubbing
    #[arg(long)]
    input_srt: Option<String>,
    /// Directory where the voice references are
    #[arg(default_value = "./temp/", long)]
    voice_refs_dir: Option<String>,
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
    let model;
    let max_tokens;
    let temperature;
    let extra_context;
    let input_srt_path;
    let output_srt_path;

    model = match options.model {
        Some(model) => model,
        None => panic!("No model for translation specified."),
    };
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

fn setup_dubber_cli(options: DubberCLI, dub_config: &mut DubConfig) {
    let llm_address = match options.address {
        Some(address) => address,
        None => panic!("No URL address for the dubber LLM connection has been specified."),
    };
    let model = match options.model {
        Some(model) => model,
        None => panic!("No model for dubbing specified."),
    };
    let wavtokenizer = match options.wavtokenizer {
        Some(model) => model,
        None => panic!("No wavtokenizer for dubbing specified."),
    };
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
    let input_srt = match options.input_srt {
        Some(audio) => audio,
        None => panic!("No input SRT file for the dubber LLM has been specified."),
    };
    let voice_refs_dir = match options.voice_refs_dir {
        Some(mut dir) => {
            // Append "/" if necessary
            if dir.chars().last() != Some('/') {
                dir.push('/');
            }
            dir
        }
        // TODO: Fix this logic... what is default value for?
        None => {
            println!("No voice references directory specified. ./temp/ will be used");
            "./temp/".to_string()
        }
    };
    let output_language = match options.output_language {
        Some(address) => address,
        None => panic!("No language to dub to specified."),
    };
    set_dubber_config(
        dub_config,
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

pub async fn setup_from_cli(dub_config: &mut DubConfig) {
    let cli = Cli::parse();
    match cli.mode {
        Mode::Translate(options) => {
            setup_translator_cli(options, dub_config);
            let srt_path = PathBuf::from(&dub_config.translator_config.input_srt_path);
            let srt_file = open_input_file(&srt_path);
            let output_srt_path = PathBuf::from(&dub_config.translator_config.output_srt_path);
            let output_srt_file = open_output_file(&output_srt_path);
            let srt_fragments = get_srt_fragments(&srt_file);
            translate_srt_file(
                &srt_fragments,
                &dub_config.translator_config,
                &output_srt_file,
            )
            .await;
            koboldcpp_start(
                &"chat".to_string(),
                &"localhost".to_string(),
                &5001,
                &dub_config.translator_config.model,
                &"".to_owned(),
                &dub_config.dubber_config.voice_refs_dir,
            )
            .await;
        }
        Mode::Dub(options) => {
            setup_dubber_cli(options, dub_config);
            let srt_path = PathBuf::from(&dub_config.dubber_config.input_srt);
            let srt_file = open_input_file(&srt_path);
            let srt_fragments = get_srt_fragments(&srt_file);
            let voice_refs = create_voice_references(
                &srt_fragments,
                &dub_config.dubber_config.input_audio,
                &dub_config.dubber_config.voice_refs_dir,
            );
            println!("Voice references: {:#?}", voice_refs);
            koboldcpp_start(
                &"tts".to_string(),
                &"localhost".to_string(),
                &5001,
                &dub_config.dubber_config.model,
                &dub_config.dubber_config.wavtokenizer,
                &dub_config.dubber_config.voice_refs_dir,
            )
            .await;
            dub_srt_file(&srt_fragments, &dub_config.dubber_config, voice_refs).await;
        }
    }
}
