use std::fs::File;
use std::io;
use std::io::Read;

use anyhow::Result;
use zip::ZipArchive;

pub fn read_text_file(archive: &mut ZipArchive<File>, path: &str) -> Result<String> {
    let file = archive.by_name(path)?;

    return Ok(io::read_to_string(file)?);
}

pub fn read_binary_file(archive: &mut ZipArchive<File>, path: &str) -> Result<Vec<u8>> {
    let mut file = archive.by_name(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    return Ok(buffer);
}
