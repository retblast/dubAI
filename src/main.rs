use crate::cli::Cli;
use crate::cli::Mode;
use crate::cli::parse_url_address;
use crate::cli::setup_dubber_cli;
use crate::cli::setup_translator_cli;
use crate::config::DubAIConfig;
use crate::dub::create_voice_references;
use crate::dub::dub_srt_file;
use crate::srt_ops::SRTFile;
use crate::translate::translate_loaded_srt_fragments;
use clap::Parser;
use llm_connect::config::KoboldChatConfig;
use llm_connect::config::KoboldConfig;
use llm_connect::config::KoboldTTSConfig;
use llm_connect::process::koboldcpp_start;
use std::path::PathBuf;
mod cli;
mod config;
mod dub;
mod error;
// Temporarily out of the equation until I re-inspect it
//mod mix;
mod srt_ops;
mod translate;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
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
            // let srt_file = open_input_file(&srt_path);
            let srt_file = SRTFile::open(srt_path)?;
            let output_srt_path = PathBuf::from(&dubai_config.translator_config.output_srt_path);
            let (host, port) = parse_url_address(&dubai_config.translator_config.llm_address)
                .expect("Failed to parse the URL");
            let kobold_chat_config = KoboldChatConfig::new(&dubai_config.translator_config.model);
            let kobold_config = KoboldConfig::new(&host, &port, None, Some(kobold_chat_config))?;
            println!("{:?}", kobold_config);
            koboldcpp_start(&kobold_config).await?;
            let output_fragments = translate_loaded_srt_fragments(
                &srt_file.fragments,
                &dubai_config.translator_config,
            )
            .await?;
            let output_srt = SRTFile::new(output_srt_path, output_fragments);
            output_srt.write()?;
        }
        Mode::Dub(options) => {
            dubai_config.dubber_config = setup_dubber_cli(options)?;
            let srt_path = PathBuf::from(&dubai_config.dubber_config.input_srt);
            //let srt_file = open_input_file(&srt_path);
            let srt_file = SRTFile::open(srt_path)?;
            //println!("{:?}", &srt_file.fragments);
            let voice_refs = create_voice_references(
                &srt_file.fragments,
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
            println!("{:?}", kobold_tts_config);
            // TEST ONLY
            // println!("llm_address: {}", &dubai_config.dubber_config.llm_address);
            // println!(
            //     "llm_address: {:?}",
            //     parse_url_address(&dubai_config.dubber_config.llm_address)
            // );
            let kobold_config = KoboldConfig::new(&host, &port, Some(kobold_tts_config), None)?;
            //println!("Voice references: {:#?}", voice_refs);
            println!("{:?}", kobold_config);
            koboldcpp_start(&kobold_config).await?;

            dub_srt_file(
                &srt_file.fragments,
                &dubai_config.dubber_config,
                voice_refs?,
            )
            .await?;
        }
    }
    Ok(())
}
