use std::path::PathBuf;

pub use clap::Parser;

#[derive(Parser)]
#[command()]
pub struct Cli {
    /// Top level directory containing subjects' directories.
    #[arg(value_name = "DIRECTORY")]
    pub data_path: PathBuf,
}
