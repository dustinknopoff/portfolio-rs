use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::Path,
};

use maud::{html, Markup, DOCTYPE};

pub(crate) mod cache;
pub(crate) mod retrieve;
pub(crate) mod templates;
use cache::PostsDatabase;
use retrieve::{copy_resources, find_content};
use templates::layout::index;

fn main() {
    if !Path::new("public").exists() {
        DirBuilder::new().create("public").unwrap();
    }
    copy_resources("resources").unwrap();
    let mut db = PostsDatabase::default();
    let files = find_content("content/").unwrap();
    db.add_posts(&files).unwrap();
    db.write_posts_to_file(&files, "public/posts").unwrap();
    // let posts = as_posts(&files)
    //     .into_iter()
    //     .filter_map(|e| e.ok())
    //     .collect::<Vec<_>>();
    // write_posts_to_file(&posts, "public/posts").unwrap();
    // let files = to_html(&posts);
    let markup = index(&db.five_most_recent(&files));
    let mut index = File::create("public/index.html").unwrap();
    index.write_all(&markup.into_string().as_bytes()).unwrap();
}
