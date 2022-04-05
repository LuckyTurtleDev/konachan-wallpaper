use reqwest::Url;
use serde::Deserialize;

const post_per_page: u64 = 10;

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

pub async fn get_post(tags: &Vec<String>, count: u64) {
	let base_url = Url::parse("https://konachan.net/post.json?limit=10&tags=rating:safe").unwrap();
	let resp = tokio::spawn(async { reqwest::get(base_url).await.unwrap().text().await });
	println!("{}", resp.await.unwrap().unwrap());
}
