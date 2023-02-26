mod error;
mod location;
#[cfg(feature = "benchmarking")]
pub mod reader;
#[cfg(not(feature = "benchmarking"))]
mod reader;
mod span;

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    string::FromUtf8Error,
};

pub use error::*;
pub use location::Location;
pub use span::Span;
use thiserror::Error;

use crate::Value;
use reader::Input;

pub fn parse(code: &str) -> Result<Vec<Value>, Error> {
    reader::parse(Input::new(None, code))
}

#[derive(Error, Debug)]
pub enum FileParseError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Encoding(#[from] FromUtf8Error),
    #[error(transparent)]
    Parse(#[from] Error),
}

pub fn parse_from_file(path: &Path) -> Result<Vec<Value>, FileParseError> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len() as usize;
    let code = unsafe {
        let mut buf = Vec::<u8>::with_capacity(size);
        file.read_exact(std::slice::from_raw_parts_mut(buf.as_mut_ptr(), size))?;
        buf.set_len(size);
        String::from_utf8(buf)?
    };

    Ok(reader::parse(Input::new(Some(path), &code))?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_file() {
        println!("{:?}", super::parse_from_file("examples/1.zle".as_ref()));
    }
}
