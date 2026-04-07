use clap::Parser;
use llm_connect::connection::koboldcpp_start;
use llm_connect::connection::openai_tts_send_prompt;

mod file_ops;
mod srt_ops;
#[derive(Parser)]
#[command(name = "dubai")]
#[command(version, about = "AI dubbing toolbox", long_about = "To dub things.")]
struct Cli {
    /// Language to dub from (fed to the AI)
    #[arg(default_value = "English", short = 'l', long)]
    input_language: String,

    /// Language to dub to (fed to the AI)
    #[arg(default_value = "English", short = 'L', long)]
    output_language: String,
}

fn main() {}
