use anyhow::{self, bail};
use clap::Parser;
use once_cell::sync::Lazy;
use rand::{seq::SliceRandom, thread_rng};
use reqwest::Client;
use std::{
	fs,
	fs::File,
	io::{prelude::*, BufReader, Write},
	process::{exit, Command},
};
use wallpaper;
use xrandr;

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

fn download() -> anyhow::Result<()> {
	let mut image_paths = get_posts(
		&vec![
			"rating:safe".to_string(),
			"red_hair".to_string(),
			"long_hair".to_string(),
			"dress".to_string(),
			"hat".to_string(),
			"short_hair".to_string(),
		],
		10,
	);
	fs::create_dir(config::WALLPAPERS_FILE.as_path().parent().unwrap());
	fs::create_dir(config::WALLPAPERS_FOLDER.to_string());
	let mut file = File::create(config::WALLPAPERS_FILE.as_path()).unwrap();
	while !image_paths.is_empty() {
		file.write_all(image_paths.pop().unwrap().as_ref());
		file.write_all(b"\n");
	}
	Ok(())
}

fn set() -> anyhow::Result<()> {
	let mut image_paths: Vec<String> = Vec::new();
	let file = File::open(config::WALLPAPERS_FILE.as_path()).unwrap();
	let bufreader = BufReader::new(file);
	for line in bufreader.lines().enumerate() {
		image_paths.push(line.1.unwrap());
	}
	let monitors = xrandr::XHandle::open().unwrap().monitors().unwrap();
	let mut active_monitors: Vec<String> = Vec::new();
	for monitor in monitors {
		if monitor.is_automatic {
			active_monitors.push(monitor.name);
		}
	}
	let mut rng = thread_rng();
	let mut command = Command::new("xwallpaper");
	for monitor in active_monitors {
		command.args(["--output", &monitor, "--zoom", image_paths.choose(&mut rng).unwrap()]);
	}
	let status = command.spawn();
	if let Err(error) = status {
		if error.kind() == std::io::ErrorKind::NotFound {
			println!("xwallpaper not found, use wallpaper crate as fallback");
			wallpaper::set_from_path(image_paths.choose(&mut rng).unwrap());
		} else {
			bail!("Error: can not set Wallpaper with : {error}")
		}
	};
	Ok(())
}

fn main() {
	let result = match Opt::parse() {
		Opt::Download => download(),
		Opt::Set => set(),
	};
	if let Err(error) = result {
		eprintln!("{:?}", error);
		exit(1);
	}
}
