use std::path::PathBuf;

use anyhow::Result;
use roxmltree::Document;

#[derive(Debug, PartialEq)]
pub struct Container {
    pub root_files: Vec<RootFile>,
}

#[derive(Debug, PartialEq)]
pub struct RootFile {
    pub base_path: String,
    pub full_path: String,
}

impl Container {
    pub fn from(doc: &str) -> Result<Container> {
        let doc = Document::parse(doc)?;
        let root_files: Vec<RootFile> = doc
            .descendants()
            .filter_map(|node| {
                if node.has_tag_name("rootfile") && node.has_attribute("full-path") {
                    let full_path = node.attribute("full-path").unwrap().to_string();
                    let mut path_buf = PathBuf::from(&full_path);
                    path_buf.pop();
                    let base_path = path_buf.to_str().unwrap().to_string();
                    Some(RootFile {
                        base_path,
                        full_path,
                    })
                } else {
                    None
                }
            })
            .collect();

        return Ok(Container { root_files });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_container() {
        let doc = r###"
<?xml version="1.0" encoding="utf-8"?>
<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
    <rootfiles>
        <rootfile full-path="epub/content.opf" media-type="application/oebps-package+xml"/>
        <rootfile full-path="EPUB/content.opf" media-type="application/oebps-package+xml"/>
        <rootfile />
    </rootfiles>
</container>
        "###
        .trim();

        let parsed = Container::from(doc).unwrap();
        let expected = Container {
            root_files: vec![
                RootFile {
                    base_path: "epub".to_string(),
                    full_path: "epub/content.opf".to_string(),
                },
                RootFile {
                    base_path: "EPUB".to_string(),
                    full_path: "EPUB/content.opf".to_string(),
                },
            ],
        };

        assert_eq!(parsed, expected)
    }
}
