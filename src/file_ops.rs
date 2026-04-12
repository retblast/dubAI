use crate::srt_ops::SRTFragment;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub fn open_input_file(input_string_path: &PathBuf) -> File {
    let input_file = match File::open(input_string_path.as_path()) {
        Err(why) => panic!(
            "Couldn't open {} because of: {}",
            input_string_path.display(),
            why
        ),
        Ok(file) => file,
    };
    return input_file;
}

pub fn open_output_file(output_string_path: &PathBuf) -> File {
    let output_file = match File::options()
        .write(true)
        .create(true)
        .open(output_string_path.as_path())
    {
        Err(why) => panic!(
            "Couldn't open {} because of: {}",
            output_string_path.display(),
            why
        ),
        Ok(file) => file,
    };
    return output_file;
}

pub fn write_srt_file(output_fragment: &SRTFragment, output_file: File) -> () {
    let mut buffered_writer = BufWriter::new(output_file);
    match write!(
        buffered_writer,
        "{}\n{}\n{}\n\n",
        output_fragment.index, output_fragment.timing, output_fragment.line,
    ) {
        Err(why) => println!("Failed to write: {}", why),
        Ok(_) => (),
    };
}
