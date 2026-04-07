use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::thread::current;

use crate::file_ops::write_srt_file;
use llm_connect::openai_tts_send_prompt;

#[derive(Clone, Debug)]
pub struct SRTFragment {
    pub index: u32,
    pub timing: String,
    pub line: String,
}

// Reads from a buffer, returns a SRTFragment and the buffer, for the next iteration
// or smth else
pub fn get_srt_fragment(buffered_srt_file: &mut BufReader<File>) -> (SRTFragment, bool) {
    let mut finished_reading = false;
    let mut current_line = String::new();
    let mut current_index = 0;
    let mut current_timing = String::new();
    let mut finished_reading = false;

    let mut current_fragment = SRTFragment {
        index: 0,
        timing: String::new(),
        line: String::new(),
    };
    while !finished_reading {
        // clear line
        current_line.clear();
        // get a new line
        match buffered_srt_file.read_line(&mut current_line) {
            Err(why) => println!("Couldn't read: {}", why),
            Ok(1_usize..) => {}
            // Finished reading the fragment
            Ok(0_usize) => {
                finished_reading = true;
                continue;
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
                current_fragment = SRTFragment {
                    index: current_index,
                    timing: current_timing,
                    line: current_line.clone().trim().to_owned(),
                };
                // Assemble current fragment for potential future use
                current_index = 0;
                current_timing = "".to_owned();
            }
        }

        current_index = match current_line.trim().parse::<u32>() {
            // Is this the correct way to "do nothing"?
            Err(_) => current_index,
            Ok(number) => number,
        };
    }

    return (current_fragment, finished_reading);
}

pub async fn new_process_srt_file(
    input_srt_file: File,
    output_srt_file: File,
    input_language: &String,
    output_language: &String,
    extra_context: &String,
) -> () {
    let mut buffered_reader = BufReader::new(input_srt_file);
    let mut current_srt_fragment: SRTFragment;
    let mut translated_fragment: SRTFragment;
    let mut finished_reading = false;

    (current_srt_fragment, finished_reading) = get_srt_fragment(&mut buffered_reader);

    // Finished reading = finished translating
    if finished_reading {
        return;
    }

    translated_fragment = current_srt_fragment.clone();
    println!("Translating: {}", &current_srt_fragment.line);
    // TODO: implement translate_line
    // translated_fragment.line = match translate_line(
    //     input_language,
    //     output_language,
    //     extra_context,
    //     &current_srt_fragment.line,
    // )
    // .await
    // {
    //     Ok(string) => string,
    //     Err(why) => {
    //         panic!("Failed to translate the current line, {}", why);
    //     }
    // };
    write_srt_file(&translated_fragment, output_srt_file);
}

// todo: add resuming flag, and unset it post resume
//
// Traverses the file, gets srt fragments, and then sends them to process them
// shouldn't return anything
// pub async fn process_srt_file(
//     input_file: &File,
//     output_file: &File,
//     resume_index: &mut u16,
//     input_language: &String,
//     output_language: &String,
//     extra_context: &String,
// ) -> () {
//     // I want to read the file, fragment by fragment.
//     // A fragment is a portion of a line of a SRT file
//     // Index number, duration and text
//     //
//     // Process:
//     // Get fragment (index, duration, text)
//     // Process (translate) it (aka) push it to the LLM
//     // Put fragment on the output file
//     // TODO: I have to consider how the code for resuming translation
//     // will be
//     let mut buffered_reader = BufReader::new(input_file);
//     let mut current_line = String::new();
//     let mut current_index = 0;
//     let mut current_timing = String::new();
//     let mut finished_reading = false;

//     let mut current_fragment = SRTFragment {
//         index: 0,
//         timing: String::new(),
//         line: String::new(),
//     };
//     let mut translated_fragment = SRTFragment {
//         index: 0,
//         timing: String::new(),
//         line: String::new(),
//     };

//     // todo: add code to handle the resume_index
//     // We can basically skip 3 lines to get to a new index, so this can help speed things up a bit
//     if *resume_index != 0 {
//         println!("Resuming from line {}", resume_index);
//         while *resume_index != 0 {
//             match buffered_reader.read_line(&mut current_line) {
//                 Err(why) => println!("Couldn't read: {}", why),
//                 Ok(1_usize..) => {}
//                 Ok(0) => {
//                     println!(
//                         "The file finished reading before we could continue with the translation"
//                     );
//                     finished_reading = true;
//                     continue;
//                 }
//             }
//             *resume_index = *resume_index - 1;
//         }
//     }
//     while !finished_reading {
//         // clear line
//         current_line.clear();
//         // get a new line
//         match buffered_reader.read_line(&mut current_line) {
//             Err(why) => println!("Couldn't read: {}", why),
//             Ok(1_usize..) => {}
//             Ok(0) => {
//                 finished_reading = true;
//                 continue;
//             }
//         }

//         if current_index != 0 {
//             if current_timing.is_empty() {
//                 current_timing = current_line.clone().trim().to_owned();
//             } else {
//                 // TODO: add more details when in debug mode

//                 // Assemble current fragment for potential future use
//                 current_fragment = SRTFragment {
//                     index: current_index,
//                     timing: current_timing,
//                     line: current_line.clone().trim().to_owned(),
//                 };
//                 translated_fragment = current_fragment.clone();
//                 println!("Translating: {}", &current_fragment.line);
//                 translated_fragment.line = match translate_line(
//                     input_language,
//                     output_language,
//                     extra_context,
//                     &current_fragment.line,
//                 )
//                 .await
//                 {
//                     Ok(string) => string,
//                     Err(why) => {
//                         panic!("Failed to translate the current line, {}", why);
//                     }
//                 };
//                 write_srt_file(&translated_fragment, output_file);
//                 current_index = 0;
//                 current_timing = "".to_owned();
//             }
//         }

//         current_index = match current_line.trim().parse::<u32>() {
//             // Is this the correct way to "do nothing"?
//             Err(_) => current_index,
//             Ok(number) => number,
//         };
//     }
// }

// checks the progress of the translated srt
// I assume that it is well formed... so
// TODO: validation/checking of the output srt.
pub fn get_translated_srt_progress(read_buffer: &mut BufReader<&File>) -> u16 {
    let mut progress_index: u16 = 0;
    let mut finished_reading = false;
    let mut current_line = String::new();
    // should loop until EOF
    while !finished_reading {
        // clear buffer
        current_line.clear();
        match read_buffer.read_line(&mut current_line) {
            Err(why) => println!("Couldn't read: {}", why),
            Ok(1_usize..) => {}
            Ok(0) => {
                finished_reading = true;
                continue;
            }
        }

        // We only care about the index number
        progress_index = match current_line.parse::<u16>() {
            // Is this the correct way to "do nothing"?
            Err(_) => progress_index,
            Ok(number) => number,
        };
    }
    return progress_index;
}
