use anyhow::Context;
use directories::{ProjectDirs, UserDirs};
use once_cell::sync::Lazy;
use serde::{self, de, Deserialize, Serialize};
use std::{collections::BTreeSet, fs::read_to_string, num::NonZeroUsize, path::PathBuf};

const CARGO_PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("de", "lukas1818", CARGO_PKG_NAME).expect("failed to get project dirs"));
pub static STATE_PATH: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.data_dir().join("state.json"));
pub static WALLPAPERS_FOLDER: Lazy<String> = Lazy::new(|| {
	match UserDirs::new() {
		Some(user_dir) => match user_dir.picture_dir() {
			Some(dir) => dir.join(format!("{}/", CARGO_PKG_NAME)).to_path_buf(),
			None => PROJECT_DIRS.data_dir().join("wallpapers/"),
		},
		None => PROJECT_DIRS.data_dir().join("wallpapers/"),
	}
	.to_string_lossy()
	.into_owned()
});
pub static CURRENT_WALLAPER_FILE: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/tmp/current-wallpaper.txt"));
pub static CONFIG_FILE: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.config_dir().join("config.toml"));

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct Action {
	pub tags: BTreeSet<String>,
}

fn deserilize_vec_event<'de, D>(deserializer: D) -> Result<Vec<Event>, D::Error>
where
	D: de::Deserializer<'de>,
{
	let mut events: Vec<Event> = Vec::deserialize(deserializer)?;
	events.sort_by_key(|v| v.priority);
	Ok(events)
}

#[derive(Debug, Deserialize)]
pub struct Event {
	pub name: Option<String>,
	pub conditon: String,
	#[serde(default)]
	pub priority: u16,
	#[serde(flatten)]
	pub action: Action,
}

fn default_count() -> NonZeroUsize {
	NonZeroUsize::new(200).unwrap()
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
	#[serde(default = "default_count")]
	pub count: NonZeroUsize,
	#[serde(deserialize_with = "deserilize_vec_event")]
	pub events: Vec<Event>,
}

impl ConfigFile {
	pub fn load() -> anyhow::Result<Self> {
		Ok(toml::from_str(
			&read_to_string(&*CONFIG_FILE)
				.with_context(|| format!("failed to open config file from {:?}", CONFIG_FILE.display()))?,
		)
		.with_context(|| format!("failed to parse config file {:?}", CONFIG_FILE.display()))?)
	}
}
