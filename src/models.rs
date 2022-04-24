#[derive(Debug)]
pub enum SplitErrors {
    FILE_NOT_FOUND,
    EMPTY_FILE,
    InternalError(String),
    InvalidConfig(String)
}