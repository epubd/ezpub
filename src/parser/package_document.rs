use std::collections::HashMap;

use anyhow::{anyhow, Result};
use roxmltree::Node;

const NAMESPACE_DC: &str = "http://purl.org/dc/elements/1.1/";

#[derive(Debug, PartialEq)]
pub(crate) struct PackageDocument {
    pub title: String,
    pub language: String,
    pub cover_image_path: Option<String>,
    pub spine: Vec<String>,
    pub manifest: HashMap<String, Option<String>>,
    pub toc_ncx_path: Option<String>,
    pub toc_nav_doc_path: Option<String>,
}

impl PackageDocument {
    pub fn from(doc: &str, base_path: &str) -> Result<PackageDocument> {
        let doc = roxmltree::Document::parse(doc)?;
        let package_elem = doc.root_element();

        let metadata_elem = package_elem
            .children()
            .find(|node| node.has_tag_name("metadata"))
            .ok_or(anyhow!("`metadata` node not found"))?;

        let (title, language) = Self::parse_metadata(&metadata_elem);

        let manifest_elem = package_elem
            .children()
            .find(|node| node.has_tag_name("manifest"))
            .ok_or(anyhow!("`manifest` node not found"))?;

        let (cover_image_path, toc_nav_doc_path, manifest_by_id, manifest_by_path) =
            Self::parse_manifest(&manifest_elem, base_path);

        let spine_elem = package_elem
            .children()
            .find(|node| node.has_tag_name("spine"))
            .ok_or(anyhow!("`spine` node not found"))?;

        let (ncx, spine) = Self::parse_spine(&spine_elem, &manifest_by_id);
        let toc_ncx_path = ncx.map(|ncx| manifest_by_id.get(&ncx).cloned()).flatten();

        Ok(PackageDocument {
            title,
            language,
            cover_image_path,
            spine,
            manifest: manifest_by_path,
            toc_ncx_path,
            toc_nav_doc_path,
        })
    }

    fn parse_metadata(metadata_elem: &Node) -> (String, String) {
        let title = metadata_elem
            .children()
            .find(|node| node.has_tag_name((NAMESPACE_DC, "title")))
            .map(|node| node.text().unwrap_or_default().trim())
            .unwrap_or_default()
            .to_string();

        let language = metadata_elem
            .children()
            .find(|node| node.has_tag_name((NAMESPACE_DC, "language")))
            .map(|node| node.text().unwrap_or_default().trim())
            .unwrap_or_default()
            .to_string();

        (title, language)
    }

    fn parse_manifest(
        manifest_elem: &Node,
        base_path: &str,
    ) -> (
        Option<String>,
        Option<String>,
        HashMap<String, String>,
        HashMap<String, Option<String>>,
    ) {
        let mut manifest_by_id = HashMap::new();
        let mut manifest_by_path = HashMap::new();
        let mut cover_image_path = None;
        let mut toc_nav_doc_path = None;

        for node in manifest_elem
            .children()
            .filter(|node| node.has_tag_name("item"))
        {
            if node.attribute("properties") == Some("cover-image") {
                cover_image_path = node
                    .attribute("href")
                    .map(|str| format!("{}/{}", base_path, str));
            }

            if node.attribute("properties") == Some("nav") {
                toc_nav_doc_path = node
                    .attribute("href")
                    .map(|str| format!("{}/{}", base_path, str));
            }

            if node.has_attribute("id") && node.has_attribute("href") {
                manifest_by_id.insert(
                    node.attribute("id").unwrap().to_string(),
                    format!("{}/{}", base_path, node.attribute("href").unwrap()),
                );

                manifest_by_path.insert(
                    format!("{}/{}", base_path, node.attribute("href").unwrap()),
                    node.attribute("media-type").map(|str| str.to_string()),
                );
            }
        }

        (
            cover_image_path,
            toc_nav_doc_path,
            manifest_by_id,
            manifest_by_path,
        )
    }

    fn parse_spine(
        spine_elem: &Node,
        manifest_by_id: &HashMap<String, String>,
    ) -> (Option<String>, Vec<String>) {
        let ncx = spine_elem
            .attribute("toc")
            .map(|toc| {
                if toc == "ncx" {
                    Some(String::from("ncx"))
                } else {
                    None
                }
            })
            .flatten();
        let spine = spine_elem
            .children()
            .filter(|node| {
                node.has_tag_name("itemref")
                    && node.has_attribute("idref")
                    && manifest_by_id.contains_key(node.attribute("idref").unwrap())
            })
            .map(|node| {
                manifest_by_id
                    .get(&node.attribute("idref").unwrap().to_string())
                    .unwrap()
                    .to_owned()
            })
            .collect();

        (ncx, spine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_package_document() {
        let doc = r###"
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" dir="ltr" prefix="se: https://standardebooks.org/vocab/1.0"
         unique-identifier="uid" version="3.0" xml:lang="en-US">
    <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
        <dc:identifier id="uid">url:https://standardebooks.org/ebooks/charlotte-bronte/jane-eyre</dc:identifier>
        <dc:title id="title"> Jane Eyre </dc:title>
        <dc:title id="subtitle">An Autobiography</dc:title>
        <dc:title id="fulltitle">Jane Eyre: An Autobiography</dc:title>
        <dc:language> en-GB </dc:language>
    </metadata>
    <manifest>
        <item href="css/core.css" id="core.css" media-type="text/css"/>
        <item href="images/cover.svg" id="cover.svg" media-type="image/svg+xml" properties="cover-image"/>
        <item href="text/chapter-1.xhtml" id="chapter-1.xhtml" media-type="application/xhtml+xml"/>
        <item href="text/endnotes.xhtml" id="endnotes.xhtml" media-type="application/xhtml+xml"/>
        <item href="text/preface.xhtml" id="preface.xhtml" media-type="application/xhtml+xml"/>
        <item href="toc.xhtml" id="toc.xhtml" media-type="application/xhtml+xml" properties="nav"/>
        <item href="toc.ncx" id="ncx" media-type="application/x-dtbncx+xml"/>
    </manifest>
    <spine toc="ncx">
        <itemref idref="preface.xhtml"/>
        <itemref idref="chapter-1.xhtml"/>
        <itemref idref="endnotes.xhtml"/>
    </spine>
</package>
        "###.trim();

        let base_path = "epub";

        let expected = PackageDocument {
            title: "Jane Eyre".to_string(),
            language: "en-GB".to_string(),
            cover_image_path: Some(format!("{}/{}", base_path, "images/cover.svg")),
            spine: vec![
                format!("{}/{}", base_path, "text/preface.xhtml"),
                format!("{}/{}", base_path, "text/chapter-1.xhtml"),
                format!("{}/{}", base_path, "text/endnotes.xhtml"),
            ],
            manifest: HashMap::from([
                (
                    format!("{}/{}", base_path, "css/core.css"),
                    Some(String::from("text/css")),
                ),
                (
                    format!("{}/{}", base_path, "images/cover.svg"),
                    Some(String::from("image/svg+xml")),
                ),
                (
                    format!("{}/{}", base_path, "text/chapter-1.xhtml"),
                    Some(String::from("application/xhtml+xml")),
                ),
                (
                    format!("{}/{}", base_path, "text/endnotes.xhtml"),
                    Some(String::from("application/xhtml+xml")),
                ),
                (
                    format!("{}/{}", base_path, "text/preface.xhtml"),
                    Some(String::from("application/xhtml+xml")),
                ),
                (
                    format!("{}/{}", base_path, "toc.xhtml"),
                    Some(String::from("application/xhtml+xml")),
                ),
                (
                    format!("{}/{}", base_path, "toc.ncx"),
                    Some(String::from("application/x-dtbncx+xml")),
                ),
            ]),
            toc_ncx_path: Some(format!("{}/{}", base_path, "toc.ncx")),
            toc_nav_doc_path: Some(format!("{}/{}", base_path, "toc.xhtml")),
        };

        let parsed = PackageDocument::from(&doc, base_path).unwrap();

        assert_eq!(expected, parsed)
    }
}
