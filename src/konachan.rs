use crate::{config, CLIENT};
use anyhow::{self, bail};
use camino::{Utf8Path, Utf8PathBuf};
use colored::*;
use futures_util::future::join_all;
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashSet, time::Duration};
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

pub async fn download_and_save_image(url: &str, path: &Utf8Path) -> anyhow::Result<()> {
	let resp = CLIENT.get(url).send().await?;
	if !resp.status().is_success() {
		bail!("error downloading image: {:?}", resp.status());
	}
	let image = resp.bytes().await?;
	fs::write(path, image).await.unwrap();
	println!("{}", path.to_string().green());
	Ok(())
}

pub async fn download_and_save_image_retry(url: String, path: impl AsRef<Utf8Path>) {
	let path = path.as_ref();
	loop {
		match download_and_save_image(&url, path).await {
			Ok(_) => break,
			Err(e) => eprintln!("{e}"),
		}
		eprintln!("rety in 1s");
		sleep(Duration::from_secs(1)).await;
	}
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
	let mut tags_string = "tags=rating:s+".to_string();
	for tag in tags.iter().take(4) {
		tags_string.push_str(tag);
		tags_string.push('+');
	}
	println!("download {count} images for the following tags: {:?}", tags);
	let mut base_url = Url::parse("https://konachan.net/post.json?limit=100000").unwrap();
	base_url.set_query(Some(&tags_string));
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
				if (!have_tag && !tag.starts_with('-')) || (have_tag && tag.starts_with('-')) {
					download_image = false;
					break;
				}
			}
			if download_image {
				{
					let file_name = Utf8PathBuf::from(format!(
						"{}Konachan.com - {}{}",
						config::WALLPAPERS_FOLDER.as_str(),
						post.id,
						&post.file_url[post.file_url.rfind(".").unwrap()..]
					));
					if file_name.exists() {
						println!("{}", file_name.to_string().dimmed())
					} else {
						images.push(tokio::spawn(download_and_save_image_retry(
							post.file_url.clone(),
							file_name.clone(),
						)));
					}
					files.push(file_name.to_string());
					picture_count += 1;
				}
			}
		}
		page += 1;
	}
	join_all(images).await;
	files
}
