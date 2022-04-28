use clap::Parser;
#[derive(Debug)]
pub enum SplitErrors {
    FILE_NOT_FOUND,
    EMPTY_FILE,
    InternalError(String),
    InvalidConfig(String)
}

/// Split A file into smaller files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    /// length of smaller files
    #[clap(short, long)]
    pub line_Length : Option<u32>,

    /// additional suffix for files
    #[clap(long, default_value = "")]
    pub additional_suffix : String,

    /// Name of file to be split
    pub file_name : String,

    /// size of smaller files
    #[clap(short,long)]
    pub byte_count : Option<String>


}

#[derive(Debug)]
pub enum SplitOptions {
    SplitByLines {
        line_length : u32,
        additional_suffix : String,
        file_name : String
    },
    SplitByBytes {
        byte_length : u64,
        additional_suffix : String,
        file_name : String,
        byte_unit : ByteUnit
    }
}

#[derive(Debug)]
pub enum ByteUnit {
    KB,
    MB
}

impl ByteUnit {
   pub fn parse(byte_unit_char : char) -> Result<ByteUnit,String> {
        match byte_unit_char {
            'k' => {  Ok(ByteUnit::KB)  },
            'm' => {  Ok(ByteUnit::MB)  },
            _ => Err(format!("Invalid Byte unit suffix {}",byte_unit_char))
        }
    }

}