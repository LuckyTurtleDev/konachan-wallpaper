use crate::{config, CLIENT};
use anyhow::{self, bail};
use futures_util::future::join_all;
use reqwest::Url;
use serde::Deserialize;
use std::{path::Path, time::Duration};
use tokio::{fs, time::sleep};

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

pub async fn download_and_save_image(url: String, path: impl AsRef<Path>) {
	let path = path.as_ref();
	let image = CLIENT.get(url.clone()).send().await.unwrap().bytes().await.unwrap();
	fs::write(path, image).await.unwrap();
	println!("{}", path.display());
}

async fn get_page(page: u64, base_url: &Url) -> anyhow::Result<Vec<ApiPost>> {
	println!("url: {base_url}");
	loop {
		let resp = CLIENT.get(base_url.clone()).query(&[("page", page)]).send().await?;
		if resp.status().is_success() {
			return Ok(resp.json::<Vec<ApiPost>>().await?);
		}
		if resp.status().is_client_error() {
			bail!("client error {}", resp.status());
		}
		eprintln!("error downloading page: {:?}; retry in 50ms", resp.status());
		sleep(Duration::from_millis(50)).await;
	}
}

#[tokio::main]
pub async fn get_posts(tags: &Vec<String>, count: usize) -> Vec<String> {
	let tags_string = if tags.is_empty() {
		None
	} else {
		let mut tmp = "tags=".to_string();
		for (i, tag) in tags.iter().enumerate() {
			if i > 4 {
				break;
			}
			tmp.push_str(tag);
			tmp.push('+');
		}
		Some(tmp)
	};
	let mut base_url = Url::parse("https://konachan.net/post.json?limit=100000").unwrap();
	base_url.set_query(tags_string.as_deref());
	let mut picture_count: usize = 0;
	let mut page: u64 = 1;
	let mut images = Vec::with_capacity(count);
	let mut files = Vec::with_capacity(count);
	while picture_count < count {
		let posts = get_page(page, &base_url).await.unwrap();
		for post in &posts {
			if picture_count >= count {
				break;
			}
			let file_name = format!(
				"{}Konachan.com - {}{}",
				config::WALLPAPERS_FOLDER.as_str(),
				post.id,
				&post.file_url[post.file_url.rfind(".").unwrap()..]
			);
			images.push(tokio::spawn(download_and_save_image(
				post.file_url.clone(),
				file_name.clone(),
			)));
			files.push(file_name);
			picture_count += 1;
		}
		page += 1;
	}
	join_all(images).await;
	files
}
