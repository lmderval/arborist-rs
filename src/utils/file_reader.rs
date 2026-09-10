use std::fs;
use std::path::PathBuf;

use crate::utils::error::{ErrorKind, MultiError};

pub struct FileReader;

impl FileReader {
    pub fn do_pass(file: &PathBuf, errors: &mut MultiError) -> String {
        match fs::read_to_string(file) {
            Ok(source) => source,
            Err(err) => {
                errors.push(
                    ErrorKind::Cli,
                    &format!("unable to read `{}`: {}", file.to_string_lossy(), err),
                );
                String::new()
            }
        }
    }
}
