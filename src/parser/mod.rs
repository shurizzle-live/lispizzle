mod error;
mod location;
#[cfg(bench)]
pub mod reader;
#[cfg(not(bench))]
pub(crate) mod reader;
mod span;

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    string::FromUtf8Error,
};

pub use error::*;
use im_rc::Vector;
pub use location::Location;
pub use span::Span;
use thiserror::Error;

use crate::{str_cache::StrCache, Value};
use reader::Input;

pub fn parse_with_cache(code: &str, cache: StrCache) -> Result<Vector<Value>, Error> {
    reader::parse(Input::with_cache(None, code, cache))
}

pub fn parse(code: &str) -> Result<Vector<Value>, Error> {
    parse_with_cache(code, StrCache::new())
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

pub fn parse_from_file_with_cache(
    path: &Path,
    cache: StrCache,
) -> Result<Vector<Value>, FileParseError> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len() as usize;
    let code = unsafe {
        let mut buf = Vec::<u8>::with_capacity(size);
        file.read_exact(std::slice::from_raw_parts_mut(buf.as_mut_ptr(), size))?;
        buf.set_len(size);
        String::from_utf8(buf)?
    };

    Ok(reader::parse(Input::with_cache(Some(path), &code, cache))?)
}

pub fn parse_from_file(path: &Path) -> Result<Vector<Value>, FileParseError> {
    parse_from_file_with_cache(path, StrCache::new())
}

#[cfg(test)]
mod tests {
    #[test]
    fn from_file() {
        println!("{:?}", super::parse_from_file("examples/1.zle".as_ref()));
    }
}
