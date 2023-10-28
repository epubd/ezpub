use anyhow::Result;
use regex::Regex;
use roxmltree::Node;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Toc {
    pub contents: Vec<TocNode>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct TocNode {
    pub title: String,
    pub href: Option<String>,
    pub children: Option<Vec<TocNode>>,
}

impl Toc {
    pub(crate) fn from_nav_doc(doc: &str, base_path: &str) -> Result<Toc> {
        Ok(Toc {
            contents: nav::parse(doc, base_path)?,
        })
    }

    pub(crate) fn from_ncx(_doc: &str, _base_path: &str) -> Result<Toc> {
        Ok(Toc {
            contents: Vec::new(),
        })
    }
}

mod nav {
    use anyhow::anyhow;

    use super::*;

    pub(crate) fn parse(doc: &str, base_path: &str) -> Result<Vec<TocNode>> {
        let doc = roxmltree::Document::parse(doc)?;
        let toc_node = doc
            .descendants()
            .find(|node| node.has_tag_name("nav") && node.attribute("id") == Some("toc"))
            .ok_or(anyhow!("`nav(id=toc)` node not found"))?;
        let ol_elem = toc_node
            .children()
            .find(|node| node.has_tag_name("ol"))
            .ok_or(anyhow!("top level `ol` node not found"))?;

        Ok(ol_elem
            .children()
            .filter(|node| node.has_tag_name("li"))
            .map(|node| parse_li(node, base_path))
            .collect())
    }

    fn parse_li(li_elem: Node, base_path: &str) -> TocNode {
        let a_elem = li_elem.children().find(|node| node.has_tag_name("a"));
        let (title, path) = if let Some(a_elem) = a_elem {
            (
                text_norm(&a_elem),
                a_elem
                    .attribute("href")
                    .map(|str| format!("{}/{}", base_path, str)),
            )
        } else {
            li_elem
                .children()
                .find(|node| node.has_tag_name("span"))
                .map(|li_elem| (text_norm(&li_elem), None))
                .unwrap_or((String::new(), None))
        };

        let children = li_elem
            .children()
            .find(|node| node.has_tag_name("ol"))
            .map(|ol_elem| {
                ol_elem
                    .children()
                    .filter(|node| node.has_tag_name("li"))
                    .map(|node| parse_li(node, base_path))
                    .collect()
            });

        TocNode {
            title,
            href: path,
            children,
        }
    }

    fn collect_text(node: &Node) -> String {
        if node.is_text() {
            node.text().unwrap().to_string()
        } else {
            let mut text = String::new();
            for child_node in node.children() {
                text.push_str(&collect_text(&child_node));
            }
            text
        }
    }

    fn text_norm(node: &Node) -> String {
        Regex::new(r"\s+")
            .unwrap()
            .replace_all(collect_text(node).trim(), " ")
            .to_string()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn normalize_html_text() {
            let text = "<div><span> <span>a </span>:  b</span>c</div>";
            let doc = roxmltree::Document::parse(text).unwrap();
            assert_eq!(String::from("a : bc"), text_norm(&doc.root()))
        }

        #[test]
        fn parse_nav_doc() {
            let doc = r#"
<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="en-US">
<head>
    <title>Table of Contents</title>
</head>
<body>
<nav id="toc">
    <h2>Table of Contents</h2>
    <ol>
        <li><a href="preface.xhtml">Preface</a></li>
        <li>
            <a href="title-page.xhtml">Jane Eyre</a>
            <ol>
                <li><a href="chapter-1.xhtml">Chapter 1</a></li>
                <li><a href="chapter-2.xhtml"> Chapter 2 </a></li>
                <li><a href="chapter-3.xhtml"><span>Chapter 3 </span></a></li>
                <li><a href="chapter-4.xhtml"> <span> Chapter</span> 4</a></li>
                <li><span><span>Chapter</span> 5</span></li>
            </ol>
        </li>
    </ol>
</nav>
</body>
</html>"#.trim();

            let base_path = "epub";

            let expected = vec![
                TocNode {
                    title: String::from("Preface"),
                    href: Some(format!("{}/{}", base_path, "preface.xhtml")),
                    children: None,
                },
                TocNode {
                    title: String::from("Jane Eyre"),
                    href: Some(format!("{}/{}", base_path, "title-page.xhtml")),
                    children: Some(vec![
                        TocNode {
                            title: String::from("Chapter 1"),
                            href: Some(format!("{}/{}", base_path, "chapter-1.xhtml")),
                            children: None,
                        },
                        TocNode {
                            title: String::from("Chapter 2"),
                            href: Some(format!("{}/{}", base_path, "chapter-2.xhtml")),
                            children: None,
                        },
                        TocNode {
                            title: String::from("Chapter 3"),
                            href: Some(format!("{}/{}", base_path, "chapter-3.xhtml")),
                            children: None,
                        },
                        TocNode {
                            title: String::from("Chapter 4"),
                            href: Some(format!("{}/{}", base_path, "chapter-4.xhtml")),
                            children: None,
                        },
                        TocNode {
                            title: String::from("Chapter 5"),
                            href: None,
                            children: None,
                        },
                    ]),
                },
            ];

            let parsed = parse(doc, base_path).unwrap();

            assert_eq!(expected, parsed);
        }
    }
}
