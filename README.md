Split clone written in rust.

Ended up being faster than the original

![Screenshot 2022-04-23 at 21 02 57](https://user-images.githubusercontent.com/24932953/164944411-243d901e-9c35-4aba-a4b4-0d055f641a28.png)


### USAGE 
```aidl
Split A file into smaller files

USAGE:
    split [OPTIONS] <FILE_NAME>

ARGS:
    <FILE_NAME>    Name of file to be split

OPTIONS:
        --additional-suffix <ADDITIONAL_SUFFIX>
           suffix for newly created files [default: ]

    -h, --help
            Print help information

    -l, --line-length <LINE_LENGTH>
            Create smaller files of l lines in length [default: 1000]

    -V, --version
            Print version information
```