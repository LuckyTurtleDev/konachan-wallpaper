use once_cell::sync::Lazy;
use reqwest::Client;
use std::{fs, fs::File, io::Write};

mod config;

mod konachan;
use konachan::*;

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

fn main() {
	let mut image_paths = get_posts(&vec!["rating:safe".to_string()], 10);
	fs::create_dir(config::WALLPAPERS_FILE.as_path().parent().unwrap());
	fs::create_dir(config::WALLPAPERS_FOLDER.to_string());
	let mut file = File::create(config::WALLPAPERS_FILE.as_path()).unwrap();
	while !image_paths.is_empty() {
		file.write_all(image_paths.pop().unwrap().as_ref());
		file.write_all(b"\n");
	}
}
