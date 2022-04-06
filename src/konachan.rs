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

pub async fn download_and_save_image(url: String) {
	CLIENT.post(url.clone()).send().await.unwrap().bytes().await.unwrap();
	println!("downloaded {url}");
}

pub async fn get_posts(tags: &Vec<String>, count: usize) {
	let base_url = Url::parse("https://konachan.net/post.json?limit=10&tags=rating:safe").unwrap();
	let mut picture_count: usize = 0;
	let mut page: u64 = 1;
	let mut images = Vec::with_capacity(count);
	while picture_count < count {
		let resp = CLIENT.post(base_url.clone()).query(&[("page", page)]).send().await;
		//todo ckech result
		let posts = resp.unwrap().json::<Vec<ApiPost>>().await;
		//todo ckech result
		let posts = posts.unwrap();
		for post in &posts {
			images.push(tokio::spawn(download_and_save_image(post.file_url.clone())));
			picture_count += 1;
		}
		page += 1;
	}
	join_all(images).await;
}
