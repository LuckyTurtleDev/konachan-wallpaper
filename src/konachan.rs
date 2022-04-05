use crate::CLIENT;
use futures_util::future::join_all;
use reqwest::Url;
use serde::Deserialize;

const post_per_page: usize = 10;

#[derive(Debug, Deserialize)]
struct ApiPost {
	id: u64,
	tags: String,
	file_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Post {
	pub id: u64,
	pub tags: Vec<String>,
	pub file_url: String,
}

pub async fn get_posts(tags: &Vec<String>, count: usize) {
	let base_url = Url::parse("https://konachan.net/post.json?limit=10&tags=rating:safe").unwrap();
	let mut page_count = count / post_per_page;
	if count % post_per_page != 0 {
		page_count += 1;
	}
	let mut pages = Vec::with_capacity(page_count);
	for i in 1..=page_count {
		pages.push(CLIENT.post(base_url.clone()).query(&[("page", i)]).send());
	}
	let pages = join_all(pages).await;
	for page in pages {
		println!("{}", page.unwrap().text().await.unwrap());
	}
}
