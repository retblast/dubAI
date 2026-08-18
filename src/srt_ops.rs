use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Default, Clone, Debug)]
pub struct SRTFragment {
    pub index: usize,
    pub timing: String,
    pub line: String,
}

#[derive(Debug)]
pub enum SRTError {
    // We error out at an index number
    TimingsParseError { index: usize },
}

pub fn get_srt_timings(srt_fragment: &SRTFragment) -> Result<(String, String), SRTError> {
    let (start, end) = match srt_fragment.timing.split_once("-->") {
        Some((start, end)) => (start.trim().replace(',', "."), end.trim().replace(',', ".")),
        None => {
            return Err(SRTError::TimingsParseError {
                index: srt_fragment.index,
            });
        }
    };
    return Ok((start, end));
}

// Reads from a buffer, returns a SRTFragment and the buffer, for the next iteration
// or smth else
pub fn get_srt_fragments(srt_file: &File) -> Vec<SRTFragment> {
    let mut vector_fragments = Vec::new();
    let mut buffered_srt_file = BufReader::new(srt_file);
    let mut current_line = String::new();
    let mut current_index = 0;
    let mut current_timing = String::new();

    loop {
        // clear line
        current_line.clear();
        // get a new line
        match buffered_srt_file.read_line(&mut current_line) {
            Err(why) => println!("Couldn't read: {}", why),
            Ok(1_usize..) => {}
            // Finished reading the fragment
            Ok(0_usize) => {
                return vector_fragments;
            }
        }
        // After reading the current index
        if current_index != 0 {
            // Read the timing first
            if current_timing.is_empty() {
                current_timing = current_line.clone().trim().to_owned();
            } else {
                // Finally, we also now have the current line, so
                // assemble the whole fragment
                let current_fragment = SRTFragment {
                    index: current_index,
                    timing: current_timing,
                    line: current_line.clone().trim().to_owned(),
                };
                // println!("current_index: {}", current_fragment.index);
                // println!("current_timing_pot: {}", current_fragment.timing);
                // println!("current_line: {}", current_fragment.line);

                vector_fragments.push(current_fragment);
                // Clean up for next iteration
                current_index = 0;
                current_timing = "".to_owned();
                current_line = "".to_owned();
            }
        }
        current_index = match current_line.trim().parse::<usize>() {
            // Is this the correct way to "do nothing"?
            Err(_) => current_index,
            Ok(number) => number,
        };
    }
}
