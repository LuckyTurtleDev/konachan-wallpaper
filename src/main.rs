use anyhow::{self, bail, Context};
use chrono::offset::Utc;
use clap::Parser;
use evalexpr::HashMapContext;
use more_wallpapers;
use once_cell::sync::Lazy;
use open;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
	collections::HashSet,
	fs,
	fs::{create_dir_all, File, OpenOptions},
	io::Write,
	process::exit,
};

mod config;
use config::{Action, Event, EventType};
mod context;
mod konachan;
use konachan::*;

use crate::{config::ConfigFile, context::get_context};

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[derive(Debug, Parser)]
struct OptDownload {
	/// download pictures for alle events, inculding inactivs
	#[clap(short, long)]
	all: bool,
}

#[derive(Debug, Parser)]
struct OptOpen {
	/// program, with will be used to open the config file
	#[clap(short, long)]
	with: Option<String>,
}

#[derive(Debug, Parser)]
enum Opt {
	/// download new pictures
	Download(OptDownload),

	/// set a dowloaded picture as Wallpaper
	Set,

	/// open config file at system default program
	OpenConfig(OptOpen),
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
pub struct ActionState {
	action_hash: u32,
	files: Vec<String>,
	last_update: i64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct State {
	actions: Vec<ActionState>,
}

impl State {
	fn load(allow_not_found: bool) -> anyhow::Result<Self> {
		let content = fs::read_to_string(&*config::STATE_PATH);
		let content = match content {
			Ok(value) => Ok(value),
			Err(error) => {
				if error.kind() == std::io::ErrorKind::NotFound {
					if allow_not_found {
						return Ok(Self::default());
					}
					eprintln!("run 'konachan-wallpaper download' first, to dowload wallpapers");
				}
				Err(error)
			},
		};
		let content = content.with_context(|| format!("failed to open state from {:?}", config::STATE_PATH.display()))?;
		Ok(serde_json::from_str(&content)
			.with_context(|| format!("failed to parse state from {:?}", config::STATE_PATH.display()))?)
	}

	fn save(&self) -> anyhow::Result<()> {
		fs::write(&*config::STATE_PATH, serde_json::to_string(&self).unwrap())
			.with_context(|| format!("failed to save state to {:?}", config::STATE_PATH.display()))?;
		Ok(())
	}
}

fn get_action(events: Vec<Event>, context: HashMapContext) -> Vec<Action> {
	let mut actions = Vec::new();
	for event in events {
		print!("{:<10} => ", event.name.unwrap_or("unamed Event".to_owned()));
		if evalexpr::eval_boolean_with_context(&event.conditon, &context)
			.with_context(|| format!("error evaluating conditon: {}", &event.conditon))
			.unwrap_or_else(|err| {
				eprintln!("{err}");
				false
			}) {
			println!("active       ");
			let mut new_action = event.action;
			if let Some(expr) = new_action.count_expr.as_ref() {
				let count = evalexpr::eval_int_with_context(expr, &context)
					.with_context(|| format!("error evaluating conut expressinon: {}", &event.conditon));
				match count {
					Ok(value) => new_action.count = Some(value as usize),
					Err(err) => eprintln!("{err}"),
				}
			}

			match event.event_type {
				EventType::Add => actions.push(new_action),
				EventType::Replace => {
					actions.clear();
					actions.push(new_action);
				},
				EventType::Modifi => {
					for action in actions.iter_mut() {
						action.modifi(&new_action);
					}
				},
				EventType::Copy => {
					let mut copy = actions.clone();
					for action in actions.iter_mut() {
						action.modifi(&new_action);
					}
					actions.append(&mut copy);
				},
			};
		} else {
			println!("inactive     ");
		}
	}
	if actions.is_empty() {
		eprintln!("No event is active");
		exit(-1);
	}
	actions
}

fn download(opt: OptDownload) -> anyhow::Result<()> {
	create_dir_all(&*config::WALLPAPERS_FOLDER)?;
	create_dir_all(config::STATE_PATH.as_path().parent().unwrap())?;
	let config = ConfigFile::load()?;
	let context = get_context(config.wifi_scan)?;
	let actions = if opt.all {
		config.events.iter().map(|event| event.action.clone()).collect()
	} else {
		get_action(config.events, context)
	};
	let mut state = State::load(true)?;
	for action in actions {
		let hash = action.get_hash();
		let image_paths = get_posts(
			&action.tags.into_iter().collect(),
			action.count.unwrap_or(config.count.into()),
		);
		println!("{} images were dowloaded", image_paths.len());
		let mut found = false;
		for action_state in state.actions.iter_mut() {
			if action_state.action_hash == hash {
				action_state.files = image_paths.clone();
				action_state.last_update = Utc::now().timestamp();
				found = true;
				break;
			}
		}
		if !found {
			state.actions.push(ActionState {
				action_hash: hash,
				files: image_paths,
				last_update: Utc::now().timestamp(),
			})
		}
	}
	state.save()?;
	Ok(())
}

fn set() -> anyhow::Result<()> {
	let config = ConfigFile::load()?;
	let actions = get_action(config.events, get_context(config.wifi_scan)?);
	let state = State::load(false)?;
	let mut pictures = HashSet::new();
	for action in actions {
		let hash = action.get_hash();
		let mut found_action = false;
		for action_state in state.actions.iter() {
			if hash == action_state.action_hash {
				if action.count.unwrap_or(config.count.into()) > action_state.files.len() {
					eprintln!(
						"Warning: need more wallpaper for action. Use only {} wallpapers, need {}.",
						action_state.files.len(),
						action.count.unwrap_or(config.count.into())
					);
					eprintln!("run 'konachan-wallpaper to dowload more wallpapers");
				}
				for (i, picture) in action_state.files.iter().enumerate() {
					pictures.insert(picture);
					if i >= action.count.unwrap_or(config.count.into()) {
						break;
					}
				}
				found_action = true;
			}
		}
		if !found_action {
			eprintln!("no image dowloaded for this action. Action will be ignored. \nrun 'konachan-wallpaper download' first, to dowload wallpapers");
		}
	}
	if pictures.is_empty() {
		bail!("no image dowloaded");
	}
	let mut used_images =
		more_wallpapers::set_random_wallpapers_from_vec(pictures.iter().collect(), more_wallpapers::Mode::Crop).to_ah()?;

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

fn open_config(opt: OptOpen) -> anyhow::Result<()> {
	create_dir_all(config::CONFIG_FILE.as_path().parent().unwrap())?;
	OpenOptions::new().create(true).write(true).open(&*config::CONFIG_FILE)?; //touch file
	match opt.with {
		Some(with) => {
			println!("open {:?} with {:?}", config::CONFIG_FILE.display(), with);
			open::with(&*config::CONFIG_FILE, with)?;
		},
		None => {
			println!("open {:?} with system default progrm", config::CONFIG_FILE.display());
			open::that(&*config::CONFIG_FILE)?;
		},
	}

	Ok(())
}

fn main() {
	let result = match Opt::parse() {
		Opt::Download(opt) => download(opt),
		Opt::Set => set(),
		Opt::OpenConfig(opt) => open_config(opt),
	};
	if let Err(error) = result {
		eprintln!("ERROR: {:?}", error);
		exit(1);
	}
}
