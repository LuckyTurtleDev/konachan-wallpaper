use adler::Adler32;
use anyhow::{self, bail, Context};
use chrono::offset::Utc;
use clap::Parser;
use evalexpr::HashMapContext;
use more_wallpapers;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
	fs,
	fs::{create_dir_all, File},
	hash::Hash,
	io::Write,
	process::exit,
};

mod config;
use config::{Action, Event};
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
	files: Vec<String>,
	last_update: i64,
}

fn get_action(events: Vec<Event>, context: HashMapContext) -> anyhow::Result<Option<Action>> {
	let mut action = None;
	for event in events {
		if evalexpr::eval_boolean_with_context(&event.conditon, &context)
			.with_context(|| format!("error evaluating conditon: {}", &event.conditon))?
		{
			action = Some(event.action);
			break;
		}
	}
	Ok(action)
}

fn download() -> anyhow::Result<()> {
	create_dir_all(&*config::WALLPAPERS_FOLDER)?;
	create_dir_all(config::STATE_PATH.as_path().parent().unwrap())?;
	let context = get_context()?;
	let config: ConfigFile = toml::from_str(&read_to_string(&*config::CONFIG_FILE)?)?;
	let action = get_action(config.events, context)?.expect("No event is active");
	let mut hasher = Adler32::new();
	action.hash(&mut hasher);
	let hash = hasher.checksum();
	let image_paths = get_posts(&action.tags.into_iter().collect(), 10);
	println!("{} images were dowloaded", image_paths.len());
	let states: anyhow::Result<Vec<State>> = (|| {
		let content = fs::read_to_string(&*config::STATE_PATH);
		let content = match content {
			Ok(value) => value,
			Err(error) => {
				if error.kind() == std::io::ErrorKind::NotFound {
					return anyhow::Result::Ok(Vec::new());
				}
				return anyhow::Result::Err(error.into());
			},
		};
		anyhow::Result::Ok(serde_json::from_str(&content)?)
	})();
	let mut states: Vec<State> = states.unwrap_or_else(|error| {
		println!("Error reading {}\n{}", config::STATE_PATH.to_string_lossy(), error);
		Vec::new()
	});
	for state in states.iter_mut() {
		if state.action_hash == hash {
			state.files = image_paths.clone();
			state.last_update = Utc::now().timestamp();
		}
	}
	fs::write(&*config::STATE_PATH, serde_json::to_string(&states)?)?;
	Ok(())
}

fn set() -> anyhow::Result<()> {
	let config: ConfigFile = toml::from_str(&read_to_string(&*config::CONFIG_FILE)?)?;
	let action = get_action(config.events, get_context()?)?.expect("No event is active");
	let file_content = &fs::read_to_string(&*config::STATE_PATH);
	let file_content = match file_content {
		Ok(value) => value,
		Err(error) => {
			if error.kind() == std::io::ErrorKind::NotFound {
				bail!(
					"Error: could not open {:?}: {}\nrun 'konachan-wallpaper download' first, to dowload wallpapers",
					config::STATE_PATH.display(),
					error
				);
			}
			bail!("Error: could not open {:?}: {}", config::STATE_PATH.display(), error);
		},
	};
	let states: Vec<State> = serde_json::from_str(file_content)?;
	let mut hasher = Adler32::new();
	action.hash(&mut hasher);
	let hash = hasher.checksum();
	let mut pictures = None;
	for state in states {
		if hash == state.action_hash {
			pictures = Some(state.files);
		}
	}
	let pictures = pictures
		.expect("no image dowloaded for this action. \nrun 'konachan-wallpaper download' first, to dowload wallpapers");
	let mut used_images = more_wallpapers::set_random_wallpapers_from_vec(pictures, more_wallpapers::Mode::Crop).to_ah()?;

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
