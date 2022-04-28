mod models;

use std::env::Args;
use std::fmt::format;
use models::{SplitErrors, ByteUnit, SplitOptions, SplitOptions::*, Config };
use clap::Parser;

use std::io::{BufReader, BufRead, Error, ErrorKind, BufWriter, Write, Read};
use std::fs::{File, read};
use std::num::ParseIntError;
use std::process;

fn main() {
    let config = Config::parse();
    let file_name = config.file_name.clone();
    validate_and_parse(config).and_then(|splitOptions| {
        call_appropriate_function(splitOptions)
    }).unwrap_or_else(|e|print_error(e, file_name))
}

fn print_error(error : SplitErrors, file_name: String) -> () {
    match error {
        SplitErrors::FILE_NOT_FOUND => {
            println!("File {} not found",file_name)
        }
        SplitErrors::EMPTY_FILE => {
            println!("File {} is empty",file_name)
        }

        SplitErrors::InternalError(err) => {
            eprintln!("Error splitting file:  {:?}",err)
        }
        SplitErrors::InvalidConfig(err) => {
            println!("Invalid Config: {}",err)
        }
    }
}


fn call_appropriate_function(splitOptions : SplitOptions) -> Result<(),SplitErrors> {
    match splitOptions {
        SplitOptions::SplitByLines{ line_length, additional_suffix, file_name} => {
            let mut is_empty = true;

            let mut file_number: u32 = 0;
            let mut cursor: u32 = 0;

            let file = File::open(&file_name)?;


            let mut reader = BufReader::new(file);
            let mut writer = BufWriter::new(create_new_file(file_number, &additional_suffix)?);

            fn write_to_writer(line: std::io::Result<String>, mut writer: &mut BufWriter<File>,cursor : &mut u32) -> std::io::Result<usize> {
                let mut correct_line = line.map_err(|e| {
                    eprintln!("Cannot read line number {}", cursor);
                    e
                })?;
                correct_line.push('\n');
                *cursor +=1;
                writer.write(correct_line.as_bytes())
            }

            for line in reader.lines(){
                is_empty = false;

                if cursor == line_length {
                    // create new file
                    file_number += 1;
                    cursor = 0;


                    // forcefully flush, so contents in buffer is written before changing underlying file
                    if let Err(e) =  writer.flush() {
                        return create_split_error("Cannot Flush buffer",e)
                    }
                    let mut underlying_file = writer.get_mut();

                    if let Err(e) = create_new_file(file_number, &additional_suffix) {
                        return create_split_error("Cannot create new file",e)
                    }
                    match create_new_file(file_number, &additional_suffix) {
                        Ok(new_file) => {
                            *underlying_file = new_file

                        }
                        Err(e) => {
                            return create_split_error("Cannot create new file",e)
                        }
                    }

                    if let Err(e) = { write_to_writer(line, &mut writer,&mut cursor) } {
                        return create_split_error("Cannot Write to buffer",e)
                    }
                } else {
                    if let Err(e) = { write_to_writer(line, &mut writer,&mut cursor) } {
                        return create_split_error("Cannot Write to buffer",e)
                    }
                }
            }

            if is_empty {
                return Err(SplitErrors::EMPTY_FILE)
            } else {
                Ok(())
            }
        },
        SplitOptions::SplitByBytes { byte_length, additional_suffix,file_name, byte_unit } => {
            let mut is_empty = true;
            let mut file_number: u32 = 0;
            let mut cursor: u32 = 0;
            let mut count = 0;
            let mut reached_EOF = false;

            if let Some(num_iterations) = match byte_unit {
                ByteUnit::KB => Some(byte_length),
                ByteUnit::MB => byte_length.checked_mul(1000)
            } {

                let file = File::open(&file_name)?;


                let mut reader = BufReader::new(file);
                let mut writer = BufWriter::new(create_new_file(file_number, &additional_suffix)?);

                // 1KB Buffer
                let mut buffer = [0_u8; 1024];
                loop {
                    count +=1;
                    let mut nread = 0usize;
                    while nread < buffer.len() {
                        match reader.read(&mut buffer[nread..]) {
                            Ok(0) =>  {
                                if is_empty {
                                    return Err(SplitErrors::EMPTY_FILE);
                                } else {
                                    if reached_EOF {
                                        return Ok(())
                                    } else {
                                        reached_EOF = true;
                                        break;
                                    }
                                }
                            },
                            Ok(n) => {
                                is_empty = false;
                                nread += n ;
                            },
                            Err(e) => {
                                return Err(SplitErrors::InternalError(format!("Cannot read file : {:?}", e)));
                            }
                        }
                    }

                    if  count > num_iterations {
                        file_number += 1;
                        count = 1;
                        // forcefully flush, so contents in buffer is written before changing underlying file
                        if let Err(e) =  writer.flush() {
                            return create_split_error("Cannot Flush buffer",e)
                        }
                        let mut underlying_file = writer.get_mut();

                        if let Err(e) = create_new_file(file_number, &additional_suffix) {
                            return create_split_error("Cannot create new file",e)
                        }
                        match create_new_file(file_number, &additional_suffix) {
                            Ok(new_file) => {
                                *underlying_file = new_file

                            }
                            Err(e) => {
                                return create_split_error("Cannot create new file",e)
                            }
                        }
                        writer.write_all(&buffer[..nread])?;
                    } else {
                        writer.write_all(&buffer[..nread])?;
                    }
                }
            } else {
                return Err(SplitErrors::InvalidConfig("Byte count too large".to_string()))
            }
        }

    }

}

fn validate_and_parse(config : Config) -> Result<SplitOptions,SplitErrors> {
    // cannot supply l and b args
    if let (Some(l), Some(b)) = (config.line_Length.as_ref(),config.byte_count.as_ref()) {
        return Err(SplitErrors::InvalidConfig(format!("Cannot supply both line length: {} and byte count : {} ",l,b)))
    }
    // byte length args must end with a k or b, and other stuff must be a number
    if let Some(byte_count) = config.byte_count.as_ref() {
        let char_vec : Vec<char>= byte_count.to_lowercase().chars().collect();
        let byte_size_unit = char_vec[char_vec.len() - 1];
        let size = &char_vec[..char_vec.len() - 1];

        return match ByteUnit::parse(byte_size_unit) {
            Ok(byte_unit) => {
                let byte_count_string = String::from_iter(size);
                match byte_count_string.parse::<u64>() {
                    Ok(byte_length) => {
                        Ok(
                            SplitByBytes {
                                byte_length,
                                additional_suffix: config.additional_suffix,
                                file_name: config.file_name,
                                byte_unit
                            }
                        )
                    },
                    Err(e) => {
                        Err(SplitErrors::InvalidConfig(format!("Cannot parse {} as a number", byte_count_string)))
                    }
                }
            },
            Err(e) => {
                Err(SplitErrors::InvalidConfig(e))
            }
        }
    } else {
        Ok(SplitByLines {
            line_length : config.line_Length.unwrap(),
            additional_suffix: config.additional_suffix,
            file_name : config.file_name
        })
    }
}

fn create_new_file(file_number: u32, additional_suffix: &str) -> std::io::Result<File> {
    File::create(format!("part{}{}", file_number, additional_suffix))
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

