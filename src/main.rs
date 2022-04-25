mod models;

use std::env::Args;
use std::fmt::format;
use models::{SplitErrors, ByteUnit, SplitOptions, Config };
use clap::Parser;

use std::io::{BufReader, BufRead, Error, ErrorKind, BufWriter, Write};
use std::fs::{File, read};
use std::num::ParseIntError;
use std::process;
use crate::SplitOptions::SplitByBytes;

fn main() {
    let config = Config::parse();

   // println!("{:?}",config);
    let res = validate_config(config).expect("failure");

    println!("{:?}",res);
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


fn validate_config(config : Config) -> Result<SplitOptions,SplitErrors> {
    // cannot supply l and b args
    if let (Some(l), Some(b)) = (config.line_Length.as_ref(),config.byte_count.as_ref()) {
        return Err(SplitErrors::InvalidConfig(format!("Cannot supply both line length: {} and byte count : {} ",l,b)))
    }
    // byte length args must end with a k or b, and other stuff must be a number
    if let Some(byte_count) = config.byte_count.as_ref() {
        let char_vec : Vec<char>= byte_count.to_lowercase().chars().collect();
        let byte_size_unit = char_vec[char_vec.len() - 1];
        let size = &char_vec[..char_vec.len() - 1];
        if !size.iter().all(|c| c.is_numeric()) {
            return Err(SplitErrors::InvalidConfig(format!("Invalid Byte count {} ",String::from_iter(size))));
        }

        match ByteUnit::parse(byte_size_unit) {
            Ok(byte_unit) => {

                match String::from_iter(size).parse::<u64>() {
                    Ok(byte_length) => {
                        return Ok(
                            SplitByBytes {
                                byte_length,
                                //byte_length: u64::from_str(&String::from_iter(size)),
                                additional_suffix: config.additional_suffix,
                                file_name: config.file_name,
                                byte_unit
                            }
                        )
                    },
                    Err(e) => {
                        return Err(SplitErrors::InvalidConfig(e.to_string()))
                    }
                }
            },
            Err(e) => {
                return Err(SplitErrors::InvalidConfig(e))
            }
        }
        // match byte_size_unit {
        //    p @  'k'  | 'm' =>  {
        //         let size = &char_vec[..char_vec.len() - 1];
        //         if !size.iter().all(|c| c.is_numeric()) {
        //             return Err(SplitErrors::InvalidConfig(format!("Invalid Byte count {} ",String::from_iter(size))));
        //         }
        //     },
        //     other => {
        //         return Err(SplitErrors::InvalidConfig(format!("Invalid Byte count suffix {}",other)));
        //     }
        // }
    } else {
        Ok(SplitOptions::SplitByLines {
            line_length : config.line_Length.unwrap(),
            additional_suffix: config.additional_suffix,
            file_name : config.file_name
        })
    }


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

