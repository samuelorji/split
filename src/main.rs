mod models;

use std::env::Args;
use std::fmt::format;
use models::SplitErrors;
use clap::Parser;

use std::io::{BufReader, BufRead, Error, ErrorKind, BufWriter, Write};
use std::fs::{File, read};
use std::process;

fn main() {
    let config = Config::parse();

   // println!("{:?}",config);
    validate_config(config).expect("failure");
    // run(&config).unwrap_or_else(|splitError| {
    //     match splitError {
    //         SplitErrors::FILE_NOT_FOUND => {
    //             println!("File {} not found",&config.file_name)
    //         }
    //         SplitErrors::EMPTY_FILE => {
    //             println!("File {} is empty",&config.file_name)
    //         }
    //         SplitErrors::InternalError(err) => {
    //             eprintln!("Error splitting file:  {:?}",err)
    //         }
    //     }
    // })
}


fn validate_config(config : Config) -> Result<Config,SplitErrors> {
    // cannot supply l and b args
    if let (Some(l), Some(b)) = (config.line_Length.as_ref(),config.byte_cpunt.as_ref()) {
        return Err(SplitErrors::InvalidConfig(format!("Cannot supply both line length: {} and byte count : {} ",l,b)))
    }
    // byte length args must end with a k or b, and other stuff must be a number
    if let Some(byte_count) = config.byte_cpunt.as_ref() {
        let char_vec : Vec<char>= byte_count.chars().collect();
        let byte_size = char_vec[char_vec.len() - 1];
        match byte_size {
            'k' | 'K' | 'm' | 'M' =>  {
                let size = &char_vec[..char_vec.len() - 1];
                if !size.iter().all(|c| c.is_numeric()) {
                    return Err(SplitErrors::InvalidConfig(format!("Invalid Byte count {} ",String::from_iter(size))));
                }
            },
            other => {
                return Err(SplitErrors::InvalidConfig(format!("Invalid Byte count suffix {}",other)));
            }
        }
    }

    Ok(config)
}
/// Split A file into smaller files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Config {
    /// Create smaller files of l lines in length
    #[clap(short, long)]
    line_Length : Option<u32>,

    /// additional suffix for files
    #[clap(long, default_value = "")]
    additional_suffix : String,

    /// Name of file to be split
    file_name : String,

    /// Name of file to be split
    #[clap(short,long)]
    byte_count : Option<String>


}
fn run(config : &Config) -> Result<(),SplitErrors> {
    let mut is_empty = true;

    let mut file_number: u32 = 0;
    let mut cursor: u32 = 0;

    let file = File::open(&config.file_name)?;
    // default buffer reader is 8KB
    fn create_new_file(file_number: u32, additional_suffix: &str) -> std::io::Result<File> {
        File::create(format!("part{}{}", file_number, additional_suffix))
    }

    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(create_new_file(file_number, &config.additional_suffix)?);

    fn write_to_writer(line: std::io::Result<String>, mut writer: &mut BufWriter<File>,cursor_num : &u32) -> std::io::Result<usize> {
        let mut correct_line = line.map_err(|e| {
            eprintln!("Cannot read line number {}", cursor_num);
            e
        })?;
        correct_line.push('\n');
        writer.write(correct_line.as_bytes())
    }

    for line in reader.lines(){
        is_empty = false;

        if cursor == config.line_Length.unwrap() {
            // create new file
            file_number += 1;
            cursor = 0;


            // forcefully flush, so contents in buffer is written before changing underlying file
            if let Err(e) =  writer.flush() {
                return create_split_error("Cannot Flush buffer",e)
            }
            let mut underlying_file = writer.get_mut();

            if let Err(e) = create_new_file(file_number, &config.additional_suffix) {
                return create_split_error("Cannot create new file",e)
            }
            match create_new_file(file_number, &config.additional_suffix) {
                Ok(new_file) => {
                    *underlying_file = new_file

                }
                Err(e) => {
                    return create_split_error("Cannot create new file",e)
                }
            }

            if let Err(e) = {
                cursor += 1;
                write_to_writer(line, &mut writer,&cursor)
            } {
                return create_split_error("Cannot Write to buffer",e)
            }
        } else {
            if let Err(e) = {
                cursor += 1;
                write_to_writer(line, &mut writer,&cursor)
            } {
                return create_split_error("Cannot Write to buffer",e)
            }
        }

    }

    if is_empty {
        return Err(SplitErrors::EMPTY_FILE)
    } else {
        Ok(())
    }
}


fn create_split_error(error_msg : &str , error : Error) -> Result<(),SplitErrors> {
    Err(SplitErrors::InternalError(format!("{}:{:?}",error_msg, error)))
}

impl From<std::io::Error> for SplitErrors {
    fn from(value: Error) -> Self {
        match value.kind() {
            ErrorKind::NotFound => SplitErrors::FILE_NOT_FOUND,
            k =>  {
                println!("unknown error: {:?}",k);
                SplitErrors::InternalError(format!("{:?}",k))
            }
        }
    }
}

