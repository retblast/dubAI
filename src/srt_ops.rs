use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::num::ParseIntError;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Default, Clone, Debug, PartialEq, Copy)]
pub struct SRTTiming {
    pub start: Duration,
    pub end: Duration,
}

impl fmt::Display for SRTTiming {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} --> {}",
            form_timestamp_from_timing(&self.start),
            form_timestamp_from_timing(&self.end)
        )
    }
}

impl SRTTiming {
    pub fn default() -> SRTTiming {
        return SRTTiming {
            start: Duration::default(),
            end: Duration::default(),
        };
    }
    pub fn empty(&self) -> bool {
        let default = SRTTiming::default();
        if *self == default {
            return true;
        }
        return false;
    }
}

#[derive(Default, Clone, Debug)]
pub struct SRTFragment {
    pub index: usize,
    pub timing: SRTTiming,
    pub subtitle_lines: Vec<String>,
}

impl SRTFragment {
    // Returns the subtitle_lines as a single line
    pub fn get_flattened_lines(&self) -> String {
        self.subtitle_lines.join(" ")
    }
}

#[derive(Debug)]
pub enum SRTError {
    // We error out at an index number
    TimingsParseError { index: usize },
    TimestampParseError,
    IndexParseError(ParseIntError),
    MalformedBlockError,
    IoError(std::io::Error),
}

impl From<std::io::Error> for SRTError {
    fn from(error: std::io::Error) -> Self {
        SRTError::IoError(error)
    }
}

impl From<ParseIntError> for SRTError {
    fn from(error: ParseIntError) -> Self {
        SRTError::IndexParseError(error)
    }
}

impl Display for SRTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimingsParseError { index } => write!(f, "Failed to parse index at: {index} "),
            Self::IndexParseError(error) => write!(f, "Failed to parse the index, value: {error}"),
            Self::TimestampParseError => write!(f, "Failed to parse timestamp"),
            Self::MalformedBlockError => write!(f, "Malformed block"),
            Self::IoError(error) => write!(f, "SRT IO Error: {error}"),
        }
    }
}

// If I want to load an SRT file for translation, I will have to provide a filepath
// If I want to create and SRT file to store the translated subtitles, I will have to provide a filepath
//
pub struct SRTFile {
    // pub'ing vs getters... I don't know...
    pub filepath: PathBuf,
    pub fragments: Vec<SRTFragment>,
}

impl SRTFile {
    pub fn update_fragments(&mut self) -> Result<(), SRTError> {
        let input_file = File::open(&self.filepath)?;
        self.fragments = Self::get_srt_fragments(&input_file)?;
        Ok(())
    }

    // to read for translation
    pub fn open(filepath: PathBuf) -> Result<Self, SRTError> {
        let input_file = File::open(filepath.as_path())?;
        let fragments = Self::get_srt_fragments(&input_file)?;
        Ok(Self {
            filepath,
            fragments,
        })
    }

    // // to store the translation
    pub fn new(filepath: PathBuf, fragments: Vec<SRTFragment>) -> Self {
        Self {
            filepath,
            fragments,
        }
    }

    pub fn write(&self) -> Result<(), SRTError> {
        let output_file = File::create(&self.filepath)?;
        let mut buffered_writer = BufWriter::new(output_file);
        for fragment in &self.fragments {
            write!(
                buffered_writer,
                "{}\n{}\n{}\n\n",
                fragment.index,
                fragment.timing,
                fragment.get_flattened_lines(),
            )?
        }
        Ok(())
    }

    // Reads from a buffer, returns a SRTFragment and the buffer, for the next iteration
    // or smth else
    // Don't know how robust it is, or if it can handle malformed files
    pub fn get_srt_fragments(srt_file: &File) -> Result<Vec<SRTFragment>, SRTError> {
        let mut vector_fragments: Vec<SRTFragment> = Vec::new();
        let mut buffered_srt_file = BufReader::new(srt_file);
        let mut current_buffered_line = String::new();
        let mut current_subtitle_lines: Option<Vec<String>> = None;
        let mut current_index: Option<usize> = None;
        let mut current_timing: Option<SRTTiming> = None;

        loop {
            // clear buffer
            current_buffered_line.clear();

            // get a new line
            match buffered_srt_file.read_line(&mut current_buffered_line) {
                Err(error) => return Err(SRTError::IoError(error)),
                Ok(1_usize..) => {}
                // Finished reading the fragment
                Ok(0_usize) => {
                    return Ok(vector_fragments);
                }
            }

            if current_index.is_none() {
                // Get an index
                // This also used as a sync point
                current_index = Some(current_buffered_line.trim().parse::<usize>()?);
                continue;
            }

            if current_timing.is_none() {
                // Set the current timing
                current_timing = Some(parse_srt_timing(
                    current_buffered_line.trim(),
                    &current_index.unwrap(),
                )?);
                continue;
            }

            if current_buffered_line != "\n" {
                // Put the rest of the text into the vector of subtitle lines
                // TODO: Instead of flattening/replacing newlines for spaces, find a better
                // way to store this, maybe
                match &mut current_subtitle_lines {
                    Some(lines) => {
                        lines.push(current_buffered_line.clone());
                    }
                    None => {
                        let mut lines = Vec::new();
                        lines.push(current_buffered_line.clone());
                        current_subtitle_lines = Some(lines);
                    }
                }
            } else {
                vector_fragments.push(SRTFragment {
                    // TODO: This shouldn't be able to Panic, but
                    // just keep this in case
                    index: current_index.unwrap(),
                    timing: current_timing.unwrap(),
                    subtitle_lines: current_subtitle_lines.unwrap().to_owned(),
                });
                // reset rest of the values
                current_index = None;
                current_timing = None;
                current_subtitle_lines = None;
            };
        }
    }
}

pub fn form_timestamp_from_timing(timing: &Duration) -> String {
    let hours = (timing.as_secs() / 3600).to_string();
    let minutes = ((timing.as_secs() % 3600) / 60).to_string();
    let seconds = (timing.as_secs() % 3600).to_string();
    let miliseconds = timing.subsec_millis().to_string();

    let timestamp = format!("{hours:02}:{minutes:02}:{seconds:02},{miliseconds:03}");
    return timestamp;
}

fn parse_timestamp(timestamp: &str) -> Result<Duration, SRTError> {
    // format is 01:02:03,400
    let timestamp_parts: Vec<&str> = timestamp.split([':', ',']).collect();
    let mut timestamp_rev_iter = timestamp_parts.iter().rev();
    // The parsed value from start's last item times 10⁶ turns the miliseconds into nanoseconds
    let timestamp_nanos = match timestamp_rev_iter.next() {
        Some(next_item) => match next_item.parse::<u32>() {
            Ok(parsed_item) => parsed_item,
            Err(_) => {
                return Err(SRTError::TimestampParseError);
            }
        },
        None => return Err(SRTError::TimestampParseError),
    };
    // Now parse the secs for the duration
    let timestamp_secs = {
        let mut timestamp_secs_temp: u64 = 0;
        for (index, portion) in timestamp_rev_iter.enumerate() {
            // TODO: Improve the msg
            let parsed_portion = match portion.parse::<u64>() {
                Ok(parsed_portion) => parsed_portion,
                Err(_) => {
                    return Err(SRTError::TimestampParseError);
                }
            };
            match index {
                0 => timestamp_secs_temp += parsed_portion,
                1 => timestamp_secs_temp += parsed_portion * 60,
                2 => timestamp_secs_temp += parsed_portion * 3600,
                _ => unreachable!(),
            }
        }
        timestamp_secs_temp
    };
    return Ok(Duration::new(timestamp_secs, timestamp_nanos));
}

pub fn parse_srt_timing(srt_timing: &str, index: &usize) -> Result<SRTTiming, SRTError> {
    let parsed_timing = {
        let (matched_start, matched_end) = match srt_timing.split_once("-->") {
            Some((start, end)) => (start.trim().replace(',', "."), end.trim().replace(',', ".")),
            None => {
                return Err(SRTError::TimingsParseError {
                    // TODO: Is clone() ok here?
                    index: index.to_owned(),
                });
            }
        };
        SRTTiming {
            start: match parse_timestamp(&matched_start) {
                Ok(parsed_timestamp) => parsed_timestamp,
                Err(_) => {
                    return Err(SRTError::TimingsParseError {
                        index: index.to_owned(),
                    });
                }
            },
            end: match parse_timestamp(&matched_end) {
                Ok(parsed_timestamp) => parsed_timestamp,
                Err(_) => {
                    return Err(SRTError::TimingsParseError {
                        index: index.to_owned(),
                    });
                }
            },
        }
    };
    return Ok(parsed_timing);
}
