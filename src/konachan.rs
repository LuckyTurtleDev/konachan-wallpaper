use crate::{config, CLIENT};
use anyhow::{self, bail};
use futures_util::future::join_all;
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashSet, path::Path, time::Duration};
use tokio::{fs, time::sleep};

#[derive(Debug, Deserialize)]
struct Post {
	id: u64,
	#[serde(with = "serde_tags")]
	tags: HashSet<String>,
	file_url: String,
}
mod serde_tags {
	use serde::de::{Deserialize, Deserializer};
	use std::collections::HashSet;
	pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer).map(|s: String| s.split(" ").map(String::from).collect())
	}
}

pub async fn download_and_save_image(url: String, path: impl AsRef<Path>) {
	let path = path.as_ref();
	let image = CLIENT.get(url.clone()).send().await.unwrap().bytes().await.unwrap();
	fs::write(path, image).await.unwrap();
	println!("{}", path.display());
}

async fn get_page(page: u64, base_url: &Url) -> anyhow::Result<Vec<Post>> {
	loop {
		println!("get posts: {base_url} at page {page}");
		let resp = CLIENT.get(base_url.clone()).query(&[("page", page)]).send().await?;
		if resp.status().is_success() {
			return Ok(resp.json::<Vec<Post>>().await?);
		}
		if resp.status().is_client_error() {
			panic!("client error downloading page: {:?}", resp.status());
		}
		eprintln!("error downloading page: {:?}; retry in 50ms", resp.status());
		sleep(Duration::from_millis(50)).await;
	}
}

#[tokio::main]
pub async fn get_posts(tags: &HashSet<String>, count: usize) -> Vec<String> {
	let tags_string = if tags.is_empty() {
		None
	} else {
		let mut tmp = "tags=rating:s+".to_string();
		for (i, tag) in tags.iter().enumerate() {
			if i > 4 {
				break;
			}
			tmp.push_str(tag);
			tmp.push('+');
		}
		Some(tmp)
	};
	println!("download {count} images for tags {:?}", tags);
	let mut base_url = Url::parse("https://konachan.net/post.json?limit=100000").unwrap();
	base_url.set_query(tags_string.as_deref());
	let mut picture_count: usize = 0;
	let mut page: u64 = 1;
	let mut images = Vec::with_capacity(count);
	let mut files = Vec::with_capacity(count);
	while picture_count < count {
		let posts = get_page(page, &base_url).await;
		let posts = match posts {
			Ok(value) => value,
			Err(error) => {
				eprintln!("{error:?}");
				eprintln!("retry again in 10s");
				sleep(Duration::from_secs(10)).await;
				continue;
			},
		};
		if posts.is_empty() {
			println!("no (more) images for this tags are aviable.");
			break;
		}
		for post in &posts {
			if picture_count >= count {
				break;
			}
			let mut download_image = true;
			for tag in tags {
				let have_tag = post.tags.contains(tag.strip_prefix('-').unwrap_or_else(|| tag));
				if post.id == 324255 {
					println!("id {} {tag} {have_tag} all: {:?}", post.id, post.tags);
				}
				if (!have_tag && !tag.starts_with('-')) || (have_tag && tag.starts_with('-')) {
					download_image = false;
					break;
				}
			}
			if download_image {
				{
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
			}
		}
		page += 1;
	}
	join_all(images).await;
	files
}
