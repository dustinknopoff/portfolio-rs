use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::Path,
};

pub(crate) mod cache;
pub(crate) mod retrieve;
pub(crate) mod templates;
use cache::PostsDatabase;
use retrieve::{copy_resources, find_content};
use templates::{layout::index, pages::write_tags_to_file};

// TODO:
// - RSS Feed
// - Search?
// - Make config for Stuff?
// - Watch for changes?

fn main() -> Result<(), anyhow::Error> {
    if !Path::new("public").exists() {
        DirBuilder::new().create("public")?;
    }
    copy_resources("resources")?;
    let mut db = PostsDatabase::default();
    let files = find_content("content/")?;
    db.add_posts(&files)?;
    db.write_posts_to_file(&files, "public/posts")?;
    write_tags_to_file(db.get_tags(&files))?;
    let markup = index(&db.five_most_recent(&files));
    let mut index = File::create("public/index.html")?;
    index.write_all(&markup.into_string().as_bytes())?;
    Ok(())
}
