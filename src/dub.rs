use crate::config::DubberConfig;
use crate::srt_ops::SRTFragment;
use ffmpeg_sidecar::command::FfmpegCommand;
use llm_connect::connection::openai_tts_send_prompt;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use tokio::fs;

fn create_base_ffmpeg_command(audio_file: &String) -> FfmpegCommand {
    let mut ffmpeg_command = FfmpegCommand::new();
    ffmpeg_command.input(audio_file);
    ffmpeg_command.codec_audio("mp3");
    ffmpeg_command.args(["-b:a", "320k"]);
    ffmpeg_command
}
// Creates mp3 files that are dialogue, taken from the SRT file
pub fn create_voice_references(
    srt_fragments: &Vec<SRTFragment>,
    audio_file: String,
    output_folder: &String,
) -> HashMap<usize, String> {
    let mut ffmpeg_command = create_base_ffmpeg_command(&audio_file);
    let mut voice_references = HashMap::new();
    for current_srt_fragment in srt_fragments {
        let voice_ref_idx = current_srt_fragment.index;
        let (start, end) = match current_srt_fragment.timing.split_once("-->") {
            Some((start, end)) => (start.trim().replace(',', "."), end.trim().replace(',', ".")),
            None => panic!("Failed to read SRT timings at SRT index: {}", voice_ref_idx),
        };
        let mut output_filename = format!("{}_ref.wav", voice_ref_idx);
        // Insert before adding the path for ffmpeg
        voice_references.insert(voice_ref_idx, output_filename.to_string());
        output_filename.insert_str(0, output_folder);

        // let output_path = match PathBuf::from_str(&output_filename.as_str()) {
        //     Ok(path) => path,
        //     Err(why) => {
        //         panic!(
        //             "Failed to get path for {}, because of {}",
        //             &output_filename, why
        //         );
        //     }
        // };
        // Code to create the file
        ffmpeg_command.args(["-ss", format!("{}", start).as_str()]);
        ffmpeg_command.args(["-to", format!("{}", end).as_str()]);
        ffmpeg_command.output(&output_filename.as_str());

        match ffmpeg_command.spawn() {
            Ok(mut child) => match child.wait() {
                Ok(..) => println!("Created {}", &output_filename),
                Err(why) => println!(
                    "Failed to create {}_ref.wav, because of: {}",
                    voice_ref_idx, why
                ),
            },
            Err(why) => println!(
                "Failed to create {}_ref.wav, because of: {}",
                voice_ref_idx, why
            ),
        };
        ffmpeg_command = create_base_ffmpeg_command(&audio_file);
    }
    return voice_references;
}

// Dubs a line
// Creates a index_dubbed.mp3 file
pub async fn dub_line(
    dubber_config: &DubberConfig,
    output_folder: &String,
    line_to_dub: &String,
    voice_ref: &String,
) {
    // The output filename is: output_folder + the index of the voice ref + _dubbed.mp3
    // the trimming is kinda finnicky
    let voice_ref_idx = voice_ref.trim_end_matches("_ref.wav").to_string();
    let mut output_filename = {
        let mut temp_clone = voice_ref_idx.clone();
        temp_clone.push_str("_dubbed.mp3");
        temp_clone.insert_str(0, output_folder);
        temp_clone
    };
    match openai_tts_send_prompt(
        &dubber_config.llm_address,
        &output_filename,
        &"kcpp".to_string(),
        line_to_dub,
        voice_ref,
    )
    .await
    {
        Ok(_) => {
            let current_dir = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(_) => panic!("Couldn't get current directory"),
            };
            let dubbing_path = Path::new(current_dir.as_path()).join(&output_filename);
            match fs::rename(&output_filename, dubbing_path).await {
                Ok(_) => {}
                Err(why) => println!(
                    "Couldn't put the generated audio file in its folder, because {}",
                    why
                ),
            };
            println!(
                "Dubbed line {}, filename: {}",
                voice_ref_idx, &output_filename
            );
        }
        Err(why) => println!(
            "Failed to generate: {}, because of: {}",
            output_filename, why
        ),
    };
}

// Dub an SRT file
// Requires a running LLM
// WIP
pub fn dub_srt_file(
    srt_fragments: &Vec<SRTFragment>,
    dubber_config: &DubberConfig,
    voice_references: HashMap<usize, String>,
    output_folder: &String,
) {
    for current_srt_fragment in srt_fragments {
        // TODO: Do not use unwrap
        let voice_ref_idx: usize = current_srt_fragment.index;
        println!("Voice ref idx: {}", &voice_ref_idx);
        let voice_ref = match voice_references.get(&voice_ref_idx) {
            Some(string) => {
                println!("Dubbed: {}", string);
                string.to_owned()
            }
            None => {
                println!("Failed to get voice reference for index {}", voice_ref_idx);
                println!("A random voice will be generated as a result.");
                "random".to_string()
            }
        };
        println!("Voice ref file: {}", &voice_ref);
        // dub_line(
        //     dubber_config,
        //     output_folder,
        //     &current_srt_fragment.line,
        //     &voice_ref,
        // )
        // .await;
    }
}
