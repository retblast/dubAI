use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::num::ParseIntError;
use std::num::TryFromIntError;
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
pub enum Error {
    // We error out at an index number
    TimingsParseError { index: Option<usize> },
    TimestampParseError,
    IndexParseError(ParseIntError),
    //TODO: this might need to be more general
    MalformedBlockError,
    IoError(std::io::Error),
    //TODO: this might need to be more general
    ParseError(TryFromIntError),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error::IndexParseError(error)
    }
}

impl From<TryFromIntError> for Error {
    fn from(error: TryFromIntError) -> Self {
        Error::ParseError(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimingsParseError { index } => write!(f, "Failed to parse index at: {index:?} "),
            Self::IndexParseError(error) => write!(f, "Failed to parse the index, value: {error}"),
            Self::TimestampParseError => write!(f, "Failed to parse timestamp"),
            Self::MalformedBlockError => write!(f, "Malformed block"),
            Self::IoError(error) => write!(f, "SRT IO Error: {error}"),
            Self::ParseError(error) => write!(f, "Parsing error: {error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            _ => None,
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
    pub fn update_fragments(&mut self) -> Result<(), Error> {
        let input_file = File::open(&self.filepath)?;
        self.fragments = Self::get_srt_fragments(&input_file)?;
        Ok(())
    }

    // to read for translation
    pub fn open(filepath: PathBuf) -> Result<Self, Error> {
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

    pub fn write(&self) -> Result<(), Error> {
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

    // I parse a block, right, and if it's correct, I return a SRTFragment, no?
    // but a thing to take into account, should I ingest a previous_index too?
    // I think so..., could ingest it as an option?
    // I have a srt block, now the things I can check are:
    // indexes, technically they'd be the first thing we see.
    // if not, malformed
    // we can check the last line too, should be an index
    // if not, malformed
    // if i find a newline... i should consider it a properly formatted block
    // but if i do stuff like this, i would have to read the srt from the last block
    // and that'd require me to rewind the buffer a big. i don't know if it has a thing for this
    // i do know i can clone() it before read_line and restore it as needed, fwiw
    //
    // For now, let's assume we ONLY consume properly formatted SRT files
    fn parse_srt_block(srt_block: &[String]) -> Result<SRTFragment, Error> {
        let mut current_subtitle_lines: Option<Vec<String>> = None;
        let mut current_index: Option<usize> = None;
        let mut current_timing: Option<SRTTiming> = None;
        // we assume that:
        // first there's index
        // then there's timing
        // then the rest are subtitle lines
        for line in srt_block.iter() {
            if current_index.is_none() {
                current_index = Some(line.parse::<usize>()?);
                continue;
            }
            if current_timing.is_none() {
                current_timing = Some(parse_srt_timing(line, &None)?);
                continue;
            }
            match &mut current_subtitle_lines {
                Some(subtitle_lines) => subtitle_lines.push(line.to_owned()),
                None => {
                    let mut subtitle_lines = Vec::new();
                    subtitle_lines.push(line.to_owned());
                    current_subtitle_lines = Some(subtitle_lines);
                }
            };
        }
        // Theoretically safe to unwrap() since we verify beforehand
        return Ok(SRTFragment {
            index: current_index.unwrap(),
            timing: current_timing.unwrap(),
            subtitle_lines: current_subtitle_lines.unwrap(),
        });
    }

    pub fn get_srt_fragments(srt_file: &File) -> Result<Vec<SRTFragment>, Error> {
        let mut vector_fragments: Vec<SRTFragment> = Vec::new();
        let mut buffered_srt_file = BufReader::new(srt_file);
        let mut current_buffered_line = String::new();
        let mut rewind_buffer: i64;
        let mut current_block: Vec<String> = Vec::new();
        let mut current_parsed_timing: Option<SRTTiming> = None;

        loop {
            // clear buffer
            current_buffered_line.clear();
            // get a new line and store the read bytes
            rewind_buffer = match buffered_srt_file.read_line(&mut current_buffered_line) {
                Err(error) => return Err(Error::IoError(error)),
                Ok(read_bytes) => read_bytes,
            }
            .try_into()?;

            match rewind_buffer {
                0 => {
                    // pass what remains to the parser
                    // maybe contains the last subtitle block!
                    vector_fragments.push(SRTFile::parse_srt_block(&current_block)?);
                    return Ok(vector_fragments);
                }
                // Ignore newlines
                // TODO: we are assuming Linux ones!
                1 => continue,
                i64::MIN.. => {}
            };
            match parse_srt_timing(&current_buffered_line, &None) {
                Ok(timing) => {
                    // reading a timing again
                    if current_parsed_timing.is_some() {
                        // # of bytes from the last index read
                        let maybe_last_index_byte_size: i64 = match current_block.pop() {
                            Some(index) => index.len().try_into()?,
                            // TODO: Maybe we can be more descriptive next time
                            None => return Err(Error::MalformedBlockError),
                        };
                        //TODO: assuming linux line endings for now and that they exist.
                        // -(maybe_last_index + bytes we read this round + line ending value)
                        buffered_srt_file
                            .seek_relative(-(maybe_last_index_byte_size + rewind_buffer + 1))?;
                        vector_fragments.push(SRTFile::parse_srt_block(&current_block)?);
                        //cleaning
                        current_parsed_timing = None;
                        current_block = Vec::new();
                    } else {
                        current_parsed_timing = Some(timing);
                        current_block.push(current_buffered_line.trim().to_owned());
                    }
                    continue;
                }
                // we didn't get a timing back
                Err(_) => {
                    current_block.push(current_buffered_line.trim().to_owned());
                    continue;
                }
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

fn parse_timestamp(timestamp: &str) -> Result<Duration, Error> {
    // original format is 01:02:03,400
    // but uh I turn it into 01:02:03.400
    let (timestamp_hms, timestamp_millis) = timestamp
        .split_once('.')
        .ok_or(Error::TimestampParseError)?;
    let mut timestamp_hms_parts = timestamp_hms.split(':');
    if timestamp_hms_parts.clone().count() > 3 {
        return Err(Error::TimestampParseError);
    }
    let hours: u64 = timestamp_hms_parts
        .next()
        .ok_or(Error::TimestampParseError)?
        .parse()?;
    let minutes: u64 = timestamp_hms_parts
        .next()
        .ok_or(Error::TimestampParseError)?
        .parse()?;
    let seconds: u64 = timestamp_hms_parts
        .next()
        .ok_or(Error::TimestampParseError)?
        .parse()?;
    let millis: u64 = timestamp_millis.parse()?;
    Ok(Duration::from_secs(hours * 3600 + minutes * 60 + seconds) + Duration::from_millis(millis))
}

pub fn parse_srt_timing(srt_timing: &str, index: &Option<usize>) -> Result<SRTTiming, Error> {
    let parsed_timing = {
        let (matched_start, matched_end) = match srt_timing.trim().split_once("-->") {
            Some((start, end)) => (start.trim().replace(',', "."), end.trim().replace(',', ".")),
            None => {
                return Err(Error::TimingsParseError {
                    // TODO: Is clone() ok here?
                    index: index.to_owned(),
                });
            }
        };
        //TODO: think about the errors here
        SRTTiming {
            start: match parse_timestamp(&matched_start) {
                Ok(parsed_timestamp) => parsed_timestamp,
                Err(why) => {
                    return Err(Error::TimingsParseError {
                        index: index.to_owned(),
                    });
                }
            },
            end: match parse_timestamp(&matched_end) {
                Ok(parsed_timestamp) => parsed_timestamp,
                Err(why) => {
                    return Err(Error::TimingsParseError {
                        index: index.to_owned(),
                    });
                }
            },
        }
    };
    return Ok(parsed_timing);
}
