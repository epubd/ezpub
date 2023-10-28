use std::collections::HashMap;
use std::fs::File;

use anyhow::{anyhow, Result};
use zip::ZipArchive;
use serde::Serialize;

use crate::parser::container::Container;
use crate::parser::package_document::PackageDocument;
pub use crate::parser::toc::{Toc, TocNode};
use crate::util::zip_util::{read_binary_file, read_text_file};

mod container;
mod package_document;
mod toc;

const CONTAINER_PATH: &str = "META-INF/container.xml";

#[derive(Debug)]
pub struct Parser {
    archive: ZipArchive<File>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct BookMeta {
    /// book title
    pub title: String,
    /// `HashMap<resource path, Option<mime type>>`
    pub manifest: HashMap<String, Option<String>>,
    /// list of all page paths
    pub spine: Vec<String>,
    /// table of contents
    pub toc: Toc,
}

impl Parser {
    pub fn open(path: &str) -> Result<Parser> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;

        Ok(Parser { archive })
    }

    pub fn meta(&mut self) -> Result<BookMeta> {
        let container = read_text_file(&mut self.archive, CONTAINER_PATH)?;
        let container = Container::from(&container)?;
        if container.root_files.is_empty() {
            return Err(anyhow!("no `rootfile` found"));
        }

        let root_file = &container.root_files[0];
        let pkg_doc_path = &root_file.full_path;
        let pkg_doc = read_text_file(&mut self.archive, pkg_doc_path)?;
        let pkg_doc = PackageDocument::from(&pkg_doc, &root_file.base_path)?;

        let toc = if pkg_doc.toc_nav_doc_path.is_some() {
            let toc = read_text_file(&mut self.archive, &pkg_doc.toc_nav_doc_path.unwrap())?;
            Toc::from_nav_doc(&toc, &root_file.base_path)?
        } else if pkg_doc.toc_ncx_path.is_some() {
            let toc = read_text_file(&mut self.archive, &pkg_doc.toc_ncx_path.unwrap())?;
            Toc::from_ncx(&toc, &root_file.base_path)?
        } else {
            return Err(anyhow!("no toc found"));
        };

        Ok(BookMeta {
            title: pkg_doc.title,
            manifest: pkg_doc.manifest,
            spine: pkg_doc.spine,
            toc,
        })
    }

    pub fn resource(&mut self, path: &str) -> Result<Vec<u8>> {
        Ok(read_binary_file(&mut self.archive, path)?)
    }
}
