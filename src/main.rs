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
// - Search?
// - Make config for Stuff?
// - Watch for changes?

struct Pipeline {
    output_dir: &'static str,
    resource_dir: &'static str,
    content_dir: &'static str,
    db: PostsDatabase,
}

impl Pipeline {
    pub fn new(
        output_dir: &'static str,
        resource_dir: &'static str,
        content_dir: &'static str,
    ) -> Self {
        Pipeline {
            output_dir,
            resource_dir,
            content_dir,
            db: PostsDatabase::default(),
        }
    }

    pub fn build(mut self) -> Result<(), anyhow::Error> {
        if !Path::new(self.output_dir).exists() {
            log::debug!("public/ did not exist. Creating now");
            DirBuilder::new().create("public")?;
        }
        copy_resources(self.resource_dir)?;
        log::debug!("copied resources in to public/");
        let files = find_content(self.content_dir)?;
        log::debug!("Found {} markdown files in content/", files.len());
        self.db.add_posts(&files)?;
        log::debug!("imported files to salsa db");
        self.db.rss_to_file(self.db.generate_rss(&files)?)?;
        log::debug!("Generated and wrote RSS.");
        self.db.write_posts_to_file(&files, "public/posts")?;
        log::debug!("Generated and wrote posts to html files");
        write_tags_to_file(self.db.get_tags(&files))?;
        log::debug!("Created general tags page");
        let markup = index(&self.db.five_most_recent(&files));
        log::debug!("Retrieved top 5 posts for index building");
        let mut index = File::create("public/index.html")?;
        index.write_all(&markup.into_string().as_bytes())?;
        log::debug!("Created the index.html");
        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    Pipeline::new("public/", "resources/", "content/").build()?;
    Ok(())
}
