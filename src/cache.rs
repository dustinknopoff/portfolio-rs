use crate::retrieve::Post;
use std::{
    fs::{DirBuilder, File},
    io::Write,
    path::Path,
    path::PathBuf,
    sync::Arc,
};

#[salsa::query_group(ContentWatchStorage)]
trait ContentWatch: salsa::Database {
    #[salsa::input]
    fn file_path(&self, key: PathBuf) -> Arc<Post>;

    fn to_html(&self, key: PathBuf) -> Arc<String>;
}

fn to_html(db: &dyn ContentWatch, key: PathBuf) -> Arc<String> {
    // Read the input string:
    let input_string = db.file_path(key);
    Arc::new(input_string.as_html().into_string())
}

#[salsa::database(ContentWatchStorage)]
#[derive(Default)]
pub struct PostsDatabase {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for PostsDatabase {}

impl PostsDatabase {
    pub fn add_posts(&mut self, paths: &[PathBuf]) -> Result<(), anyhow::Error> {
        for path in paths.iter() {
            self.set_file_path(path.clone(), Arc::new(Post::new(path.to_path_buf())?));
        }
        Ok(())
    }

    pub fn write_posts_to_file(
        &mut self,
        paths: &[PathBuf],
        in_directory: &'static str,
    ) -> Result<(), anyhow::Error> {
        if !Path::new(in_directory).exists() {
            DirBuilder::new().recursive(true).create(in_directory)?;
        }
        for path in paths.iter() {
            let post = self.file_path(path.to_path_buf());
            let html = self.to_html(path.clone());
            let mut path = PathBuf::from(in_directory);
            let file_name = post.filename.file_name().unwrap();
            path.push(file_name);
            path.set_extension("html");
            let mut file = File::create(&path)?;
            file.write_all(html.as_bytes())?;
        }
        Ok(())
    }

    pub fn five_most_recent(&self, paths: &[PathBuf]) -> Vec<Arc<Post>> {
        let mut posts = paths
            .iter()
            .map(|path| {
                let post: Arc<Post> = self.file_path(path.to_path_buf());
                post
            })
            .collect::<Vec<_>>();
        posts.sort_by(|a, b| a.frontmatter.date.cmp(&b.frontmatter.date));
        let _ = posts.drain(5..);
        posts
    }
}
