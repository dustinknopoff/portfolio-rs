use crate::retrieve::{Post, PublicPath, SourcePath};
use std::{
    collections::HashMap,
    fs::{DirBuilder, File},
    io::Write,
    path::Path,
    path::PathBuf,
    sync::Arc,
};

#[salsa::query_group(ContentWatchStorage)]
trait ContentWatch: salsa::Database {
    #[salsa::input]
    fn file_path(&self, key: SourcePath) -> Arc<Post>;

    fn to_html(&self, key: SourcePath) -> Arc<String>;

    fn tags(&self, key: SourcePath) -> Arc<Vec<String>>;
}

fn to_html(db: &dyn ContentWatch, key: SourcePath) -> Arc<String> {
    // Read the input string:
    let input_string = db.file_path(key);
    Arc::new(input_string.as_html().into_string())
}

fn tags(db: &dyn ContentWatch, key: SourcePath) -> Arc<Vec<String>> {
    // Read the input string:
    let input_string = db.file_path(key);
    Arc::new(input_string.frontmatter.tags.clone())
}

#[salsa::database(ContentWatchStorage)]
#[derive(Default)]
pub struct PostsDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for PostsDatabase {}

impl PostsDatabase {
    pub fn add_posts(&mut self, paths: &[SourcePath]) -> Result<(), anyhow::Error> {
        for path in paths.iter() {
            self.set_file_path(path.clone(), Arc::new(Post::new(path.clone())?));
        }
        Ok(())
    }

    pub fn write_posts_to_file(
        &mut self,
        paths: &[SourcePath],
        in_directory: &'static str,
    ) -> Result<(), anyhow::Error> {
        if !Path::new(in_directory).exists() {
            DirBuilder::new().recursive(true).create(in_directory)?;
        }
        for path in paths.iter() {
            let post = self.file_path(path.clone());
            let html = self.to_html(path.clone());
            let mut path = PathBuf::from(in_directory);
            let file_name = post.filename.0.file_name().unwrap();
            path.push(file_name);
            path.set_extension("html");
            let mut file = File::create(&path)?;
            file.write_all(html.as_bytes())?;
        }
        Ok(())
    }

    pub fn five_most_recent(&self, paths: &[SourcePath]) -> Vec<Arc<Post>> {
        let mut posts = paths
            .iter()
            .map(|path| {
                let post: Arc<Post> = self.file_path(path.clone());
                post
            })
            .collect::<Vec<_>>();
        posts.sort_by(|a, b| b.frontmatter.date.cmp(&a.frontmatter.date));
        posts.into_iter().take(5).collect()
    }

    pub fn get_tags(&self, paths: &[SourcePath]) -> HashMap<String, Vec<PublicPath>> {
        let mut map: HashMap<String, Vec<PublicPath>> = HashMap::new();
        for path in paths {
            let post = self.file_path(path.clone());
            for tag in post.frontmatter.tags.iter() {
                dbg!(&tag);
                let list = map.entry(tag.clone()).or_default();
                list.push(path.to_public_path());
            }
        }
        map
    }
}
