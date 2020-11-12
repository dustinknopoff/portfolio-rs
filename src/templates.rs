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
                    content.push_str(&txt.to_string())
                }
            });
            html! {
                p {
                    (content.get(..140).unwrap_or(&content)) "..."
                }
            }
        }
    }

    pub fn preview(title: &str, tags: Option<&str>, blurb: Markup, url: &str) -> Markup {
        html! {
        a href=(url) {
            div.card {
                div.topline {
                    span.tags {
                        @if let Some(tags) = tags {
                            (tags)
                        }
                    }
                        h2 { (title)}
                    }
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
                   ( preview(&post.frontmatter.title, Some(&post.frontmatter.tags.join(", ")), html! {
                       (Blurb(&post.content))
                   }, "/posts"))
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
            }
            div.buffer {}
            div#sidebar {
                .tagline {
                    ul {
                        li {
                            a href="/tags/dev" {
                                span { "dev"}
                            }
                        }
                        li {
                            a href="/tags/design" {
                                span { "design"}
                            }
                        }
                        li {
                            a href="/tags" {
                                span { "tags"}
                            }
                        }
                    }
                    p { "rustacean, cook, and martial arts enthusiast"}
                    div {}
                }
                a href="/" {
                    img.logo src="/DK Logo.svg" { }
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
                        a href="https://dustinknopoff.dev/rss.xml" { img src="/rss.svg" {}}
                    }
                    div{}
                }
            }
        }
    }
}
