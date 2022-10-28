Split clone written in rust.

Ended up being faster than the original

![Screenshot 2022-04-23 at 21 02 57](https://user-images.githubusercontent.com/24932953/164944411-243d901e-9c35-4aba-a4b4-0d055f641a28.png)


### USAGE 
build in release mode with 
```bash
cargo build --release
```

### Examples
```bash
split -b 10k batch.txt --additional-suffix=.csv

split -l 100000 batch.csv
```
```bash
split 0.1.0
Split A file into smaller files

USAGE:
    split [OPTIONS] <FILE_NAME>

ARGS:
    <FILE_NAME>    Name of file to be split

OPTIONS:
        --additional-suffix <ADDITIONAL_SUFFIX>    additional suffix for files [default: ]
    -b, --byte-count <BYTE_COUNT>                  size of smaller files
    -h, --help                                     Print help information
    -l, --line-length <LINE_LENGTH>                length of smaller files
    -V, --version                                  Print version information
```
