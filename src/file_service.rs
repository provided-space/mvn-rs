use std::fs::File;
use std::io::{Error, Read, Write};

#[derive(Clone)]
pub struct FileService {}

impl FileService {
    pub fn new() -> FileService {
        return FileService {};
    }

    pub fn write(&self, path: &String, buf: &[u8]) -> Result<(), Error> {
        return File::create(path).and_then(|mut file| file.write_all(buf));
    }

    pub fn read(&self, path: &String) -> Result<Vec<u8>, Error> {
        return File::open(path).and_then(|mut file| {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            return Ok(buf);
        });
    }
}
