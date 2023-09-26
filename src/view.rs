mod highlight;

use crate::{
    model::article,
    ADDR
};


pub fn display_articles(
    articles: impl IntoIterator<Item = article::ListItem>
) -> String {
    let favicon = urlencoding::encode(include_str!("../assets/favicon.svg"));
    let markup = maud::html! {
        style {
            (maud::PreEscaped(include_str!("../css/articles.css")))
        }

        title { "160R blog"  }
        link rel="icon" href=(
            format!("data:image/svg+xml,{favicon}")
        ) { }

        body {

            h1 { "Welcome to 160R's blog!" }

            h2 { "Software" }

            a .software href="https://github.com/I60R/page" { "page" }

            a .software href="https://github.com/I60R/javelin" { "javelin" }

            h2 { "Articles" }

            main {

                @for article::ListItem { added, title, .. } in articles {
                    a .article href=(format!("{ADDR}/blog/{title}")) {
                        (format!("{added}  •  {title}\n"))
                    }
                }
            }
        }
    };

    markup.into_string()
}


pub fn display_article(article_item: &article::Item) -> String {
    let article_title_decoded = urlencoding::decode(&article_item.title)
        .unwrap();

    let article_body = base64::decode(&article_item.body)
        .unwrap();
    let article_body = String::from_utf8(article_body)
        .unwrap();

    let markdown_render = highlight::code_blocks(&article_body);

    let favicon = urlencoding::encode(include_str!("../assets/favicon.svg"));

    let markup = maud::html! {
        style {
            (maud::PreEscaped(include_str!("../css/article.css")))
        }

        title { (article_title_decoded) }
        link rel="icon" href=(
            format!("data:image/svg+xml,{favicon}")
        ) { }

        body {
            main {
                h1 { (article_title_decoded) }

                (maud::PreEscaped(markdown_render))
            }

            footer {
                @if !article_item.is_last {
                    a .arrow_backward href=(
                        format!("{ADDR}/blog/next/{}", article_item.id)
                    ) { "" }
                } @else {
                    a { "⏺" }
                }

                a href=(format!("{ADDR}")) { "⌂" }

                @if !article_item.is_first {
                    a .arrow_forward href=(
                        format!("{ADDR}/blog/prev/{}", article_item.id)
                    ) { "" }
                } @else {
                    a { "⏺" }
                }
           }
        }
    };

    markup.into_string()
}


pub fn admin_login() -> String {
    let favicon = urlencoding::encode(include_str!("../assets/favicon.svg"));

    let markup = maud::html! {
        style {
            (maud::PreEscaped(include_str!("../css/login.css")))
        }

        title { "Login" }
        link rel="icon" href=(
            format!("data:image/svg+xml,{favicon}")
        ) { }

        body {
            div {
                "username:"
                input { }

                "password:"
                input { }

                button {
                    "Login"
                }
            }
        }
    };

    markup.into_string()
}

pub fn admin_panel(
    articles: impl IntoIterator<Item = article::ListItem>
) -> String {
    let favicon = urlencoding::encode(include_str!("../assets/favicon.svg"));
    let markup = maud::html! {
        style {
            (maud::PreEscaped(include_str!("../css/admin.css")))
        }

        title { "160R blog: admin"  }
        link rel="icon" href=(
            format!("data:image/svg+xml,{favicon}")
        ) { }

        body {

            h2 { "Articles" }

            main {

                @for article::ListItem { added, title, .. } in articles {

                    div .list {
                        button .edit {
                            "edit"
                        }

                        a .article href=(format!("{ADDR}/blog/{title}")) {
                            (format!("{added}  •  {title}\n"))
                        }
                    }
                }
            }

            div .editor {

                div .title {

                    input .name { }

                    button .post {
                        "post"
                    }
                }

                input .content { }
            }
        }
    };

    markup.into_string()
}
