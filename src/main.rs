use anyhow::{self, bail, Context};
use clap::Parser;
use once_cell::sync::Lazy;
use rand::{seq::SliceRandom, thread_rng};
use reqwest::Client;
use std::{
	collections::HashSet,
	fs::{create_dir_all, read_to_string, File},
	io::{prelude::*, BufReader, Write},
	process::{exit, Command},
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

fn download() -> anyhow::Result<()> {
	create_dir_all(&*config::WALLPAPERS_FOLDER)?;
	create_dir_all(config::WALLPAPERS_FILE.as_path().parent().unwrap())?;
	let tags = read_to_string(config::CONFIG_FILE.as_path())
		.with_context(|| format!("Failed to open {}", config::CONFIG_FILE.display()))?;
	let tags: String = tags.strip_suffix('\n').unwrap_or(&tags).into();
	let tags: HashSet<String> = tags.split(" ").map(String::from).collect();
	let mut image_paths = get_posts(&tags, 200);
	println!("{} images were dowloaded", image_paths.len());
	let mut file = File::create(config::WALLPAPERS_FILE.as_path()).unwrap();
	while !image_paths.is_empty() {
		file.write_all(image_paths.pop().unwrap().as_ref())?;
		file.write_all(b"\n")?;
	}
	Ok(())
}

fn set_backgrund_backup(image_paths: &Vec<String>) -> anyhow::Result<Vec<String>> {
	let mut rng = thread_rng();
	let image = image_paths.choose(&mut rng).unwrap();
	wallpaper::set_from_path(image);
	Ok(vec![image.clone()])
}

#[cfg(not(target_os = "linux"))]
fn set_backgrund(image_paths: &Vec<String>) -> anyhow::Result<Vec<String>> {
	set_backgrund_backup(image_paths)
}

#[cfg(target_os = "linux")]
fn set_backgrund(image_paths: &Vec<String>) -> anyhow::Result<Vec<String>> {
	let monitors = xrandr::XHandle::open().unwrap().monitors().unwrap(); //TODO: Error Handling
	let mut active_monitors: Vec<String> = Vec::new();
	for monitor in monitors {
		if monitor.is_automatic {
			active_monitors.push(monitor.name);
		}
	}
	let mut rng = thread_rng();
	let mut command = Command::new("xwallpaper");
	let mut used_images = Vec::new();
	for monitor in active_monitors {
		let image = image_paths.choose(&mut rng).unwrap();
		command.args(["--output", &monitor, "--zoom", image]);
		used_images.push(image.clone());
	}
	let status = command.spawn();
	if let Err(error) = status {
		if error.kind() == std::io::ErrorKind::NotFound {
			println!("xwallpaper not found, use wallpaper crate as fallback");
			return set_backgrund_backup(image_paths);
		} else {
			bail!("Error: can not set Wallpaper with xwallpaper: {error}")
		}
	};
	Ok(used_images)
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
	let mut used_images = set_backgrund(&image_paths)?;
	println!("set {:?} as wallpaper(s)", used_images);
	if config::CURRENT_WALLAPER_FILE.parent().unwrap().is_dir() {
		println!(
			"save list of current wallpapers in {}",
			config::CURRENT_WALLAPER_FILE.as_path().display()
		);
		let mut file = File::create(config::CURRENT_WALLAPER_FILE.as_path())?;
		while !used_images.is_empty() {
			file.write_all(used_images.pop().unwrap().as_ref())?;
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
		eprintln!("{:?}", error);
		exit(1);
	}
}
