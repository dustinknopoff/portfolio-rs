use crate::templates::{layout::layout, markdown::Markdown};
use chrono::prelude::*;
use maud::{html, Markup};
use serde::Deserialize;
use std::{fmt, fs::File, io::Read, io::Write, path::PathBuf};
use walkdir::{DirEntry, WalkDir};
#[derive(Debug, Clone)]
pub struct Post {
    pub frontmatter: FrontMatter,
    pub content: String,
    pub filename: SourcePath,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FrontMatter {
    pub tags: Vec<String>,
    pub title: String,
    pub link: String,
    #[serde(deserialize_with = "from_frontmatter")]
    pub date: NaiveDateTime,
    #[serde(rename = "linkText")]
    pub link_text: Option<String>,
}

use serde::de;

struct NaiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string represents chrono::NaiveDateTime")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // 2018-06-26 08:31
        match NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
            Ok(t) => Ok(t),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
        }
    }
}

fn from_frontmatter<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(NaiveDateTimeVisitor)
}

impl Post {
    pub fn new(filename: SourcePath) -> Result<Self, anyhow::Error> {
        let mut file = File::open(filename.0.as_path())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let (matter, content) =
            frontmatter::split_matter(&content).unwrap_or((String::new(), content.clone()));
        let matter = serde_yaml::from_str(&matter)?;
        Ok(Self {
            frontmatter: matter,
            content,
            filename,
        })
    }

    pub fn as_html(&self) -> Markup {
        layout(
            &self.frontmatter.title,
            html! {
                h1 { (self.frontmatter.title)}
                article {
                    (Markdown(&self.content))
                }
            },
        )
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_md(entry: &DirEntry) -> bool {
    entry.path().extension().map(|s| s == "md").unwrap_or(false)
}

pub fn find_content(location: &'static str) -> Result<Vec<SourcePath>, anyhow::Error> {
    Ok(WalkDir::new(location)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| is_md(e))
        .map(|entry| SourcePath(entry.into_path()))
        .collect::<Vec<_>>())
}

pub fn copy_resources(location: &'static str) -> Result<(), anyhow::Error> {
    for entry in WalkDir::new(location)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            let mut new_path = PathBuf::from("public");
            new_path.push(entry.path().file_name().unwrap());
            let mut old_file = File::open(entry.path())?;
            let mut old_contents = Vec::new();
            old_file.read_to_end(&mut old_contents)?;
            let mut new_file = File::create(&new_path)?;
            new_file.write_all(&old_contents)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourcePath(pub PathBuf);

impl SourcePath {
    pub fn to_public_path(&self) -> PublicPath {
        let mut new_path = PathBuf::from("public");
        new_path.push(self.0.file_name().unwrap());
        new_path.set_extension("html");
        PublicPath(new_path)
    }
}

impl AsRef<PathBuf> for SourcePath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsMut<PathBuf> for SourcePath {
    fn as_mut(&mut self) -> &mut PathBuf {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicPath(PathBuf);

impl AsRef<PathBuf> for PublicPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsMut<PathBuf> for PublicPath {
    fn as_mut(&mut self) -> &mut PathBuf {
        &mut self.0
    }
}

impl PublicPath {
    #[allow(dead_code)]
    pub fn to_src_path(&self) -> SourcePath {
        let mut new_path = PathBuf::from("content");
        new_path.push(self.0.file_name().unwrap());
        SourcePath(new_path)
    }
}
