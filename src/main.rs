use anyhow::{self, bail, Context};
use clap::Parser;
use more_wallpapers;
use once_cell::sync::Lazy;
use reqwest::Client;
use std::{
	collections::HashSet,
	fs::{create_dir_all, read_to_string, File},
	io::{prelude::*, BufReader, Write},
	process::exit,
};

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
	println!("load config from {:?}", config::CONFIG_FILE.display());
	create_dir_all(&*config::WALLPAPERS_FOLDER)?;
	create_dir_all(config::WALLPAPERS_FILE.as_path().parent().unwrap())?;
	let tags = read_to_string(config::CONFIG_FILE.as_path())
		.with_context(|| format!("Failed to open {}", config::CONFIG_FILE.display()))?;
	let tags: String = tags.strip_suffix('\n').unwrap_or(&tags).into();
	let mut tags: HashSet<String> = tags.split(" ").map(String::from).collect();
	tags.remove("");
	let mut image_paths = get_posts(&tags, 200);
	println!("{} images were dowloaded", image_paths.len());
	let mut file = File::create(config::WALLPAPERS_FILE.as_path()).unwrap();
	while !image_paths.is_empty() {
		file.write_all(image_paths.pop().unwrap().as_ref())?;
		file.write_all(b"\n")?;
	}
	Ok(())
}

fn set() -> anyhow::Result<()> {
	let mut image_paths: Vec<String> = Vec::new();
	let file = File::open(config::WALLPAPERS_FILE.as_path());
	let file = match file {
		Ok(value) => value,
		Err(error) => {
			if error.kind() == std::io::ErrorKind::NotFound {
				bail!(
					"Error: could not open {:?}: {}\nrun 'konachan-wallpaper download' first, to dowload wallpapers",
					config::WALLPAPERS_FILE.display(),
					error
				);
			}
			bail!("Error: could not open {:?}: {}", config::WALLPAPERS_FILE.display(), error);
		},
	};
	let bufreader = BufReader::new(file);
	for line in bufreader.lines().enumerate() {
		image_paths.push(line.1.unwrap());
	}
	if image_paths.is_empty() {
		bail!(
			"Error {:?} is empty\nrun 'konachan-wallpaper download', to download wallpapers",
			config::WALLPAPERS_FILE.display()
		);
	}
	let mut used_images = more_wallpapers::set_random_wallpapers_from_vec(image_paths, more_wallpapers::Mode::Crop)?;

	println!("set {:?} as wallpaper(s)", used_images);
	if config::CURRENT_WALLAPER_FILE.parent().unwrap().is_dir() {
		println!(
			"save list of current wallpapers in {}",
			config::CURRENT_WALLAPER_FILE.as_path().display()
		);
		let mut file = File::create(config::CURRENT_WALLAPER_FILE.as_path())?;
		while let Some(path) = used_images.pop() {
			file.write_all(path.to_str().unwrap().as_ref())?;
			file.write_all(b"\n")?;
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
		eprintln!("ERROR: {:?}", error);
		exit(1);
	}
}
