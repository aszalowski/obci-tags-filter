use std::{
    borrow::Cow,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use walkdir::{DirEntry, WalkDir};

const TAG_FILE_PREFIX: &str = "N400_";
const EXCLUDE_WORDS_FILE_PREFIX: &str = "exclude_words_";
const EXCLUDE_WORDS_FILE_EXTENSION: &str = ".txt";
pub const TAG_FILE_EXTENSION: &str = ".obci.tag";

fn is_tag_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .is_some_and(|s| s.starts_with(TAG_FILE_PREFIX) && s.ends_with(TAG_FILE_EXTENSION))
}

pub fn get_input_path_prefix(input_path: &Path) -> Result<Cow<'_, str>> {
    Ok(input_path
        .file_prefix()
        .with_context(|| format!("invalid input path: {}", input_path.display()))?
        .to_string_lossy())
}

pub fn make_exclude_words_path(input_path: &Path) -> Result<PathBuf> {
    let middle = get_input_path_prefix(input_path)?
        .strip_prefix(TAG_FILE_PREFIX)
        .with_context(|| format!("{} has invalid name.", input_path.display()))?
        .to_string();

    Ok(input_path.with_file_name(format!(
        "{}{}{}",
        EXCLUDE_WORDS_FILE_PREFIX, middle, EXCLUDE_WORDS_FILE_EXTENSION
    )))
}

pub fn make_output_path(input_path: &Path) -> Result<PathBuf> {
    let input_path_prefix = get_input_path_prefix(input_path)?;

    Ok(input_path.with_file_name(format!(
        "{}_excluded{}",
        input_path_prefix, TAG_FILE_EXTENSION
    )))
}

pub fn make_backup_path(input_path: &Path) -> Result<PathBuf> {
    let input_path_prefix = get_input_path_prefix(input_path)?;

    Ok(input_path.with_file_name(format!(
        "{}_backup{}",
        input_path_prefix, TAG_FILE_EXTENSION
    )))
}

pub fn load_exclude_words<R>(exclude_words_reader: R) -> Result<Vec<String>>
where
    R: BufRead,
{
    let exclude_words: Result<_, std::io::Error> = exclude_words_reader.lines().collect();
    Ok(exclude_words?)
}

pub fn load_exclude_words_from_file<P>(exclude_words_path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = File::open(exclude_words_path.as_ref()).with_context(|| {
        format!(
            "can't open file {}",
            exclude_words_path.as_ref().display()
        )
    })?;
    load_exclude_words(BufReader::new(file))
}

pub fn find_tag_files<P>(root_dir: P) -> impl Iterator<Item = Result<DirEntry, walkdir::Error>>
where
    P: AsRef<Path>,
{
    let walker = WalkDir::new(root_dir).min_depth(2).max_depth(2);
    walker.into_iter().filter_entry(is_tag_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_path_from_input_path() {
        let input_path = PathBuf::from(r"test_data/p6/N400_ab6c_PW2.obci.tag");
        let output_path = make_output_path(&input_path);
        assert_eq!(
            output_path.unwrap().to_str().unwrap(),
            r"test_data/p6/N400_ab6c_PW2_excluded.obci.tag"
        )
    }

    #[test]
    fn exclude_words_path_from_input_path() {
        let input_path = PathBuf::from(r"test_data/p6/N400_ab6c_PW2.obci.tag");
        let exclude_words_path = make_exclude_words_path(&input_path);
        assert_eq!(
            exclude_words_path.unwrap().to_str().unwrap(),
            r"test_data/p6/exclude_words_ab6c_PW2.txt"
        )
    }
}
