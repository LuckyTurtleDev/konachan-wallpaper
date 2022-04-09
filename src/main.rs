use clap::Parser;
use once_cell::sync::Lazy;
use rand::{seq::SliceRandom, thread_rng};
use reqwest::Client;
use std::{
	fs,
	fs::File,
	io::{prelude::*, BufReader, Write},
};
use wallpaper;

mod config;

mod konachan;
use konachan::*;

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(Debug, Parser)]
enum Opt {
	/// download new pictures
	Download,

	/// set a dowloaded picture as Wallpaper
	Set,
}

fn download() {
	let mut image_paths = get_posts(&vec!["rating:safe".to_string()], 10);
	fs::create_dir(config::WALLPAPERS_FILE.as_path().parent().unwrap());
	fs::create_dir(config::WALLPAPERS_FOLDER.to_string());
	let mut file = File::create(config::WALLPAPERS_FILE.as_path()).unwrap();
	while !image_paths.is_empty() {
		file.write_all(image_paths.pop().unwrap().as_ref());
		file.write_all(b"\n");
	}
}

fn set() {
	let mut image_paths: Vec<String> = Vec::new();
	let file = File::open(config::WALLPAPERS_FILE.as_path()).unwrap();
	let bufreader = BufReader::new(file);
	for line in bufreader.lines().enumerate() {
		image_paths.push(line.1.unwrap());
	}
	let mut rng = thread_rng();
	wallpaper::set_from_path(image_paths.choose(&mut rng).unwrap());
}

fn main() {
	match Opt::parse() {
		Opt::Download => download(),
		Opt::Set => set(),
	};
}
