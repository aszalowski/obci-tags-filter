#![feature(path_file_prefix)]

use anyhow::{bail, Result};
use std::{fs::{self, File}, io::BufReader};
use walkdir::DirEntry;

use xml_processing::process_xml;

use crate::{cli::{Cli, Parser}, io::make_backup_path};
use crate::io::{
    find_tag_files, load_exclude_words_from_file, make_exclude_words_path, make_output_path,
};

mod cli;
mod io;
mod xml_processing;

fn process_entry(entry: &DirEntry) -> Result<()> {
    println!("Found tag file: {}", entry.file_name().to_string_lossy());
    let input_path = entry.path();
    let exclude_words_path = make_exclude_words_path(input_path)?;
    let exclude_words = load_exclude_words_from_file(exclude_words_path)?;

    let input = BufReader::new(File::open(input_path)?);

    let output_path = make_output_path(input_path)?;
    let mut output = File::create(output_path.clone())?;

    process_xml(input, &mut output, exclude_words.as_slice());

    fs::rename(input_path, make_backup_path(input_path)?)?;
    fs::rename(output_path, input_path)?;

    println!("Processed.\n");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    for entry in find_tag_files(cli.data_path) {
        match entry {
            Ok(entry) => {
                if let Err(error) = process_entry(&entry) {
                    println!(
                        "Skipping {}: {}\n",
                        entry.file_name().to_string_lossy(),
                        error.to_string()
                    )
                }
            }
            Err(error) => bail!("{:?}", error),
        }
    }

    println!("Finished.");
    Ok(())
}
