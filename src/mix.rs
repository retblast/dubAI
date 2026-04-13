use crate::srt_ops::SRTFragment;
use crate::srt_ops::get_srt_timings;
use ffmpeg_sidecar::command::FfmpegCommand;
use std::collections::HashMap;
use std::convert;
use std::process::Command;
use std::time::Duration;

fn convert_string_to_duration(duration: String) -> Result<Duration, Box<dyn std::error::Error>> {
    let time_units: Vec<&str> = {
        let mut temp_vec: Vec<&str> = duration
            .trim()
            .rsplit(|pattern| pattern == ':' || pattern == '.')
            .collect();
        temp_vec.reverse();
        temp_vec
    };
    println!("{:?}", time_units);
    let mut seconds = 0;
    for time_unit in &time_units[0..2] {
        seconds += match time_unit.parse::<usize>() {
            Ok(value) => value,
            Err(why) => panic!("Failed to convert in c_s_t_d: {}", why),
        };
        seconds *= 60;
    }
    // Add the seconds
    seconds += match time_units[2].parse::<usize>() {
        Ok(value) => value,
        Err(why) => panic!("Failed to convert in c_s_t_d: {}", why),
    };
    let nanos = match time_units[3].parse::<usize>() {
        // nano -> micro
        Ok(value) => value * 1000,
        Err(why) => panic!("Failed to convert in c_s_t_d: {}", why),
    };

    // TODO: handle this gracefully
    let duration = Duration::new(seconds.try_into().unwrap(), nanos.try_into().unwrap());
    Ok(duration)
}

fn calculate_subtitle_duration(start: Duration, end: Duration) -> Duration {
    return end - start;
}

pub fn get_dubbed_audio_duration(dubbed_audio: &String) -> Duration {
    let mut ffprobe_command = Command::new("ffprobe");
    ffprobe_command
        .args(["-v", "error"])
        .arg("-sexagesimal")
        .args(["-show_entries", "format=duration"])
        .args(["-of", "default=noprint_wrappers=1:nokey=1"])
        .arg(dubbed_audio);

    let ffprobe_output = {
        let output = ffprobe_command
            .output()
            .expect(&format!(
                "Couldn't get the duration from ffprobe of {dubbed_audio}"
            ))
            .stdout;
        String::from_utf8(output).expect("Couldn't convert the ffprobe output to a String")
    };
    let dubbed_audio_duration =
        convert_string_to_duration(ffprobe_output).expect("Failed to convert string to duration");

    return dubbed_audio_duration;
}

pub fn calculate_duration_ratio(
    srt_fragment: &SRTFragment,
    dubbed_audio: &String,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let (start, end) = get_srt_timings(srt_fragment);
    let start_duration = convert_string_to_duration(start)?;
    let end_duration = convert_string_to_duration(end)?;
    let actual_duration = calculate_subtitle_duration(start_duration, end_duration);
    let dubbed_audio_duration = 0;

    return Ok(actual_duration);
}
// pub fn prepare_fragments(srt_fragments: Vec<SRTFragment>, voice_refs: HashMap<usize, String>) {}
