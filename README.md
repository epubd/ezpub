# ezpub ![workflow](https://github.com/epubd/ezpub/actions/workflows/rust.yml/badge.svg)

An EPUB parser written in Rust, with support for both EPUB 2.0 and 3.0 versions.

## install
`cargo add --git https://github.com/epubd/ezpub.git`

## example

```rust
fn main() {
    let mut parser = ezpub::parser::Parser::open("sample.epub").unwrap();

    let book_meta = parser.meta().unwrap();
    println!("{:?}", book_meta);

    let resource_path = "epub/toc.xhtml";
    let resource = parser.resource(resource_path).unwrap();
    println!("{:?}", resource);
}
```

## features

`serde`: `derive(Serialize, Deserialize)`

## structs

```rust
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
```

```rust
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Toc {
    pub contents: Vec<TocNode>,
}
```

```rust
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TocNode {
    pub title: String,
    pub href: Option<String>,
    pub children: Option<Vec<TocNode>>,
}
```