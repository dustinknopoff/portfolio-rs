pub mod markdown {
    use maud::{html, Markup, PreEscaped, Render};
    use pulldown_cmark::{html as c_html, CodeBlockKind, CowStr, Event, Parser, Tag};
    use syntect::highlighting::ThemeSet;
    use syntect::html::highlighted_html_for_string;
    use syntect::parsing::SyntaxSet;

    /// Renders a block of Markdown using `pulldown-cmark`.
    pub struct Markdown<T: AsRef<str>>(pub T);

    impl<T: AsRef<str>> Render for Markdown<T> {
        fn render(&self) -> Markup {
            // Generate raw HTML
            let mut unsafe_html = String::new();
            let mut lang = String::new();
            let mut in_code_block = false;
            let parser = Parser::new(self.0.as_ref()).map(|event| match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    if let CodeBlockKind::Fenced(ref attr) = kind {
                        lang = attr.to_string();
                        in_code_block = true;
                    }
                    Event::Start(Tag::CodeBlock(kind))
                }
                Event::Text(ref text) | Event::Html(ref text) if in_code_block => {
                    Event::Html(CowStr::Boxed(Box::from(highlighted_html_for_language(
                        &text.to_string(),
                        lang.clone(),
                    ))))
                }
                Event::End(Tag::CodeBlock(kind)) => {
                    in_code_block = false;
                    Event::End(Tag::CodeBlock(kind))
                }
                _ => event,
            });
            c_html::push_html(&mut unsafe_html, parser);
            // Sanitize it with ammonia
            let safe_html = ammonia::Builder::default()
                .add_generic_attributes(&["class", "style"])
                .clean(&unsafe_html)
                .to_string();
            // let safe_html = ammonia::clean(&unsafe_html);
            PreEscaped(safe_html)
        }
    }

    fn highlighted_html_for_language(snippet: &str, attributes: String) -> String {
        lazy_static::lazy_static! {
            static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
            static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
        };
        let theme = &THEME_SET.themes["base16-ocean.dark"];
        let syntax = SYNTAX_SET
            .find_syntax_by_token(&attributes)
            .or_else(|| SYNTAX_SET.find_syntax_by_name(&attributes))
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());
        highlighted_html_for_string(snippet, &SYNTAX_SET, syntax, theme)
    }

    /// Renders a block of Markdown using `pulldown-cmark`.
    pub struct Blurb<T: AsRef<str>>(pub T);

    impl<T: AsRef<str>> Render for Blurb<T> {
        fn render(&self) -> Markup {
            // Generate raw HTML
            let mut content = String::new();
            Parser::new(self.0.as_ref()).for_each(|x| {
                if let Event::Text(ref txt) = x {
                    content.push_str(&txt.to_string());
                    content.push(' ');
                }
            });
            html! {
                p {
                    (content.get(..140).unwrap_or(&content)) "..."
                }
            }
        }
    }

    pub fn preview(title: &str, tags: Option<&[String]>, blurb: Markup, url: &str) -> Markup {
        html! {
            div.card {
                div.topline {
                    ul.tags {
                        @if let Some(tags) = tags {
                            @for tag in tags {
                                a href={"/tags/" (tag) ".html"} {
                                    li { (tag)}
                                }
                            }
                        }
                    }
                    a href={"/" (url)} {
                        h2 { (title)}
                    }
                }
                a href={"/" (url)} {
                    (blurb)
                }
            }
        }
    }
}

pub mod css {
    use maud::{html, Markup, Render};

    /// Links to a CSS stylesheet at the given path.
    pub struct Css(pub &'static str);

    impl Render for Css {
        fn render(&self) -> Markup {
            html! {
                link rel="stylesheet" type="text/css" href=(self.0);
            }
        }
    }
}

pub mod layout {
    use std::sync::Arc;

    use crate::retrieve::Post;

    use super::{
        css::Css,
        markdown::{preview, Blurb},
    };
    use maud::{html, Markup, DOCTYPE};

    pub fn index(five_recent: &[Arc<Post>]) -> Markup {
        layout(
            "Dustin Knopoff",
            html! {
                @for post in five_recent {
                   ( preview(&post.frontmatter.title, Some(&post.frontmatter.tags), html! {
                       (Blurb(&post.content))
                   }, post.filename.to_public_path(false).0.to_str().unwrap()))
                }
            },
        )
    }

    pub fn layout(title: &str, content: Markup) -> Markup {
        html! {
            (DOCTYPE)
            head {
                title {(title)}
                (Css("/reset.css"))
                (Css("/style.css"))
                meta charset="utf-8";
                (favicons())
            }
            div.buffer {}
            div#sidebar {
                .tagline {
                    ul {
                        li {
                            a href="/tags/dev.html" {
                                span { "dev"}
                            }
                        }
                        li {
                            a href="/tags/design.html" {
                                span { "design"}
                            }
                        }
                        li {
                            a href="/tags/tags.html" {
                                span { "tags"}
                            }
                        }
                    }
                    a href="/posts/about.html" { "rustacean, cook, and martial arts enthusiast"}
                    div {}
                }
                a.logo href="/" {
                    img src="/DK Logo.png" { }
                }
            }
            div#main {
                main {
                    (content)
                }
                footer {
                    span{"Copyright 2020"}
                    div.links {
                        a onClick="
                        javascript:window.location.href=atob('bWFpbHRvOndlYi1jb250YWN0QGtub3BvZmYuZGV2')"  { img src="/mail.svg" {}}
                        a href="https://github.com/dustinknopoff" rel="noopener noreferrer nofollow" target="_blank"{ img src="/github.svg" {}}
                        a href="https://linkedin.com/in/dustinknopoff" { img src="/linkedin.svg" {}}
                        a href="/feed.xml" { img src="/rss.svg" {}}
                    }
                    div{}
                }
            }
        }
    }

    fn favicons() -> Markup {
        html! {
            link rel="apple-touch-icon" sizes="57x57" href="/apple-icon-57x57.png" {}
            link rel="apple-touch-icon" sizes="60x60" href="/apple-icon-60x60.png" {}
            link rel="apple-touch-icon" sizes="72x72" href="/apple-icon-72x72.png" {}
            link rel="apple-touch-icon" sizes="76x76" href="/apple-icon-76x76.png" {}
            link rel="apple-touch-icon" sizes="114x114" href="/apple-icon-114x114.png" {}
            link rel="apple-touch-icon" sizes="120x120" href="/apple-icon-120x120.png" {}
            link rel="apple-touch-icon" sizes="144x144" href="/apple-icon-144x144.png" {}
            link rel="apple-touch-icon" sizes="152x152" href="/apple-icon-152x152.png" {}
            link rel="apple-touch-icon" sizes="180x180" href="/apple-icon-180x180.png" {}
            link rel="icon" type="image/png" sizes="192x192"  href="/android-icon-192x192.png" {}
            link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" {}
            link rel="icon" type="image/png" sizes="96x96" href="/favicon-96x96.png" {}
            link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png" {}
            link rel="manifest" href="/manifest.json" {}
            meta name="msapplication-TileColor" content="#ffffff" {}
            meta name="msapplication-TileImage" content="/ms-icon-144x144.png" {}
            meta name="theme-color" content="#ffffff" {}
        }
    }
}

pub mod pages {
    use std::{
        collections::HashMap,
        fs::{DirBuilder, File},
        io::Write,
        path::Path,
        path::PathBuf,
        sync::Arc,
    };

    use maud::{html, Markup};

    use super::{
        layout::layout,
        markdown::{preview, Blurb},
    };
    use crate::retrieve::Post;

    pub fn write_tags_to_file(
        tag_map: HashMap<String, Vec<Arc<Post>>>,
    ) -> Result<(), anyhow::Error> {
        if !Path::new("public/tags").exists() {
            DirBuilder::new().recursive(true).create("public/tags")?;
        }
        let keys = tag_map.keys().collect::<Vec<_>>();
        let all_tags = tags(&keys);
        {
            let mut file = File::create("public/tags/tags.html")?;
            file.write_all(all_tags.into_string().as_bytes())?;
        }
        for key in keys {
            let tag_html = tag(key, &tag_map[key]);
            {
                let mut path = PathBuf::from("public/tags");
                path.push(key);
                path.set_extension("html");
                let mut file = File::create(path)?;
                file.write_all(tag_html.into_string().as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn tags(tags: &[&String]) -> Markup {
        layout(
            "Dustin Knopoff | Tags",
            html! {
                h3 { "Tags"}
                @for tag in tags {
                    a href={"/tags/" (tag) ".html"} {
                        li { (tag)}
                    }
                }
            },
        )
    }

    pub fn tag(tag: &str, posts: &[Arc<Post>]) -> Markup {
        layout(
            &format!("Dustin Knopoff | {}", tag),
            html! {
                h3 { (tag)}
                @for post in posts {
                   ( preview(&post.frontmatter.title, Some(&post.frontmatter.tags), html! {
                       (Blurb(&post.content))
                   }, post.filename.to_public_path(false).0.to_str().unwrap()))
                }
            },
        )
    }
}
