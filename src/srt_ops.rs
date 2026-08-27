use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
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
    pub subtitle_lines: String,
}

#[derive(Debug)]
pub enum SRTError {
    // We error out at an index number
    TimingsParseError { index: usize },
    TimestampParseError,
    IndexParseError,
    MalformedBlockError,
    IoError(std::io::Error),
}

impl From<std::io::Error> for SRTError {
    fn from(error: std::io::Error) -> Self {
        SRTError::IoError(error)
    }
}

impl Display for SRTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimingsParseError { index } => write!(f, "Failed to parse index at: {index} "),
            Self::IndexParseError => write!(f, "Failed to parse the index"),
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
    filepath: PathBuf,
    fragments: Vec<SRTFragment>,
}

impl SRTFile {
    // to read for translation
    pub fn open(filepath: PathBuf) -> Result<Self, SRTError> {
        let input_file = File::open(filepath.as_path())?;
        let fragments = get_srt_fragments(&input_file)?;
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
                fragment.index, fragment.timing, fragment.subtitle_lines,
            )?
        }
        Ok(())
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
                    index: index.clone(),
                });
            }
        };
        SRTTiming {
            start: match parse_timestamp(&matched_start) {
                Ok(parsed_timestamp) => parsed_timestamp,
                Err(_) => {
                    return Err(SRTError::TimingsParseError {
                        index: index.clone(),
                    });
                }
            },
            end: match parse_timestamp(&matched_end) {
                Ok(parsed_timestamp) => parsed_timestamp,
                Err(_) => {
                    return Err(SRTError::TimingsParseError {
                        index: index.clone(),
                    });
                }
            },
        }
    };
    return Ok(parsed_timing);
}

// Reads from a buffer, returns a SRTFragment and the buffer, for the next iteration
// or smth else
pub fn get_srt_fragments(srt_file: &File) -> Result<Vec<SRTFragment>, SRTError> {
    let mut vector_fragments: Vec<SRTFragment> = Vec::new();
    let mut buffered_srt_file = BufReader::new(srt_file);
    let mut current_buffered_line = String::new();
    let mut current_subtitle_lines = String::new();
    let mut current_index = 0;
    let mut current_timing = SRTTiming::default();

    loop {
        // clear buffer
        current_buffered_line.clear();

        // get a new line
        match buffered_srt_file.read_line(&mut current_buffered_line) {
            //TODO: fix
            Err(_) => return Err(SRTError::MalformedBlockError),
            Ok(1_usize..) => {}
            // Finished reading the fragment
            Ok(0_usize) => {
                return Ok(vector_fragments);
            }
        }

        // If the index isn't set, and the current timing is empty
        // we can fill the timing
        if current_index != 0 {
            if current_timing.empty() {
                // Set the current timing
                current_timing =
                    match parse_srt_timing(current_buffered_line.trim(), &current_index) {
                        Ok(parsed_timing) => parsed_timing,
                        Err(_) => {
                            return Err(SRTError::TimingsParseError {
                                index: current_index,
                            });
                        }
                    };
            } else {
                // If we find a pure newline, that means we finished reading a block
                if current_buffered_line != "\n" {
                    // Put the rest of the text into the vector of subtitle lines
                    // TODO: Instead of flattening/replacing newlines for spaces, find a better
                    // way to store this, maybe
                    current_subtitle_lines.push_str(&current_buffered_line);
                    current_subtitle_lines.push(' ');
                } else {
                    vector_fragments.push(SRTFragment {
                        index: current_index,
                        timing: current_timing,
                        subtitle_lines: current_subtitle_lines.trim().to_owned(),
                    });
                    // reset rest of the values
                    current_timing = SRTTiming::default();
                    current_index = 0;
                };
            }
        } else {
            // Get an index
            // This also used as a sync point
            current_index = match current_buffered_line.trim().parse::<usize>() {
                // Is this the correct way to "do nothing"?
                Err(_) => return Err(SRTError::IndexParseError),
                Ok(number) => number,
            };
        }
    }
}
