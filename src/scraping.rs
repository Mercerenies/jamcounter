
use scraper::{Html, Selector, ElementRef};

#[derive(Debug, Clone)]
pub struct ForumPost {
  pub author: String,
  pub text: String,
}

pub async fn read_page(url: &str) -> reqwest::Result<Html> {
  let body = reqwest::get(url).await?.text().await?;
  let html = Html::parse_document(&body);
  Ok(html)
}

pub fn parse_posts(html: &Html) -> Vec<Option<ForumPost>> {
  let selector = Selector::parse(".message-inner").unwrap();
  html.select(&selector)
    .map(parse_post)
    .collect()
}

fn parse_post(post: ElementRef) -> Option<ForumPost> {
  let author_elt = post.select(&Selector::parse(".message-name").ok()?).next()?;
  let content_elt = post.select(&Selector::parse(".message-cell--main").ok()?).next()?;
  Some(ForumPost {
    author: author_elt.text().collect(),
    text: content_elt.text().collect(),
  })
}

pub async fn read_and_parse_page(url: &str) -> reqwest::Result<Vec<Option<ForumPost>>> {
  let html = read_page(url).await?;
  Ok(parse_posts(&html))
}
