use std::path::PathBuf;

pub struct Config {
    pub input_file: PathBuf,
    pub out_file: PathBuf,
    pub out_format: Format,
    pub regions_csv: PathBuf,
}

pub enum Format {
}
