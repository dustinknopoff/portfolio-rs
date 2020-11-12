use crate::{
    retrieve::{Post, SourcePath},
    templates::markdown::{Blurb, Markdown},
};
use anyhow::anyhow;
use chrono::Local;
use maud::html;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
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

    pub fn get_tags(&self, paths: &[SourcePath]) -> HashMap<String, Vec<Arc<Post>>> {
        let mut map: HashMap<String, Vec<Arc<Post>>> = HashMap::new();
        for path in paths {
            let post = self.file_path(path.clone());
            for tag in post.frontmatter.tags.iter() {
                let list = map.entry(tag.clone()).or_default();
                list.push(post.clone());
            }
        }
        map
    }

    pub fn generate_rss(&self, paths: &[SourcePath]) -> Result<Channel, anyhow::Error> {
        let mut items: Vec<Item> = Vec::new();
        for path in paths.iter() {
            let post = self.file_path(path.clone());
            let link = format!(
                "https://dustinknopoff.dev/{}",
                post.filename.to_public_path(false).0.to_str().unwrap()
            );
            items.push(
                ItemBuilder::default()
                    .title(post.frontmatter.title.clone())
                    .link(link)
                    .pub_date(post.frontmatter.date.to_rfc2822())
                    .description(html! {(Blurb(&post.content))}.into_string())
                    .content(html! {(Markdown(&post.content))}.into_string())
                    .build()
                    .map_err(|x| anyhow!("{}", x))?,
            );
        }
        let namespaces = {
            let mut namespaces = HashMap::new();
            namespaces.insert(
                String::from("atom"),
                String::from("http://www.w3.org/2005/Atom"),
            );
            namespaces.insert(
                String::from("dc"),
                String::from("http://purl.org/dc/elements/1.1/"),
            );
            namespaces.insert(
                String::from("content"),
                String::from("http://purl.org/rss/1.0/modules/content/"),
            );
            namespaces
        };
        use rss::validation::Validate;
        let channel = ChannelBuilder::default()
            .title("Dustin Knopoff")
            .link("http://dustinknopoff.dev")
            .description("rustacean, cook, and martial arts enthusiast")
            .last_build_date(Local::now().to_rfc2822())
            .items(items)
            .namespaces(namespaces)
            .build()
            .map_err(|x| anyhow!("{}", x))?;
        channel.validate()?;
        Ok(channel)
    }

    pub fn rss_to_file(&self, channel: Channel) -> Result<(), anyhow::Error> {
        let mut file = File::create("public/feed.xml")?;
        file.write_all(channel.to_string().as_bytes())?;
        Ok(())
    }
}
