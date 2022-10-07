use adler::Adler32;
use anyhow::{self, bail, Context};
use clap::Parser;
use more_wallpapers;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
	fs::{create_dir_all, File},
	hash::Hash,
	io::{prelude::*, BufReader, Write},
	process::exit,
};

mod config;
use config::Action;
mod context;
mod konachan;
use konachan::*;
mod utils;
use utils::read_to_string;

use crate::{config::ConfigFile, context::get_context};

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(Debug, Parser)]
enum Opt {
	/// download new pictures
	Download,

	/// set a dowloaded picture as Wallpaper
	Set,
}

trait BoxedErrorHandling<V, E>
where
	E: std::fmt::Display,
{
	fn to_ah(self) -> anyhow::Result<V>;
}

impl<V, E> BoxedErrorHandling<V, E> for Result<V, E>
where
	E: std::fmt::Display,
{
	fn to_ah(self) -> anyhow::Result<V> {
		match self {
			Ok(value) => Ok(value),
			Err(error) => bail!("{error}"),
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct State {
	action_hash: u32,
	last_update: i64,
}

fn download() -> anyhow::Result<()> {
	create_dir_all(&*config::WALLPAPERS_FOLDER)?;
	create_dir_all(config::STATE_PATH.as_path().parent().unwrap())?;
	let context = get_context()?;
	let config: ConfigFile = toml::from_str(&read_to_string(&*config::CONFIG_FILE)?)?;
	let mut action = None;
	for event in config.events {
		if evalexpr::eval_boolean_with_context(&event.conditon, &context)
			.with_context(|| format!("error evaluating conditon: {}", &event.conditon))?
		{
			action = Some(event.action);
			break;
		}
	}
	let action = action.expect("No event is active");
	let mut hasher = Adler32::new();
	serde_json::to_string(&action)?.hash(&mut hasher); // Hashset does not impl Hash
	hasher.checksum();
	let mut state: State = serde_json::from_str(&read_to_string(&*config::STATE_PATH)?)?;
	let mut image_paths = get_posts(&action.tags, 200);
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
	let mut used_images =
		more_wallpapers::set_random_wallpapers_from_vec(image_paths, more_wallpapers::Mode::Crop).to_ah()?;

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
