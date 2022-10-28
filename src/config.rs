use adler::Adler32;
use anyhow::Context;
use directories::{ProjectDirs, UserDirs};
use once_cell::sync::Lazy;
use serde::{self, de, Deserialize, Serialize};
use std::{collections::BTreeSet, fs::read_to_string, hash::Hash, num::NonZeroUsize, path::PathBuf};
use strum_macros::{Display, EnumString};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Action {
	pub tags: BTreeSet<String>,
	pub ratio: Option<f64>,
	pub ratio_expr: Option<String>,
}

impl Action {
	pub fn modifi(&mut self, action: &Action) {
		self.tags.extend(action.tags.clone());
		if action.ratio.is_some() {
			self.ratio = action.ratio;
		}
	}

	//we do only need the hast of some fields
	pub fn get_hash(&self) -> u32 {
		let mut hasher = Adler32::new();
		self.tags.hash(&mut hasher);
		hasher.checksum()
	}
}

pub trait VecAction {
	fn normalize_to(&mut self, target_ratio: f64);
	fn normilize(&mut self) {
		self.normalize_to(1.0);
	}
}

impl VecAction for Vec<Action> {
	fn normalize_to(&mut self, target_ratio: f64) {
		let sum: f64 = self.iter().map(|a: &Action| a.ratio.unwrap_or(1.0)).sum();
		for action in self.iter_mut() {
			action.ratio = Some((action.ratio.unwrap_or(1.0) / sum) * target_ratio);
		}
	}
}

fn deserilize_vec_event<'de, D>(deserializer: D) -> Result<Vec<Event>, D::Error>
where
	D: de::Deserializer<'de>,
{
	let mut events: Vec<Event> = Vec::deserialize(deserializer)?;
	events.sort_by_key(|v| v.priority);
	Ok(events)
}

#[derive(Clone, Debug, Default, Deserialize, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum EventType {
	#[default]
	/// Repclace all active events
	Replace,
	/// add a new event
	Add,
	/// modifi all active events
	Modifi,
	/// copy active action and modifi the copy.
	//Did not change existing events
	Copy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
	#[serde(default, rename = "type")]
	pub event_type: EventType,
	pub name: Option<String>,
	pub conditon: String,
	#[serde(default)]
	pub priority: u16,
	#[serde(default)]
	pub force_ratio: bool,
	#[serde(flatten)]
	pub action: Action,
}

fn default_count() -> NonZeroUsize {
	NonZeroUsize::new(200).unwrap()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
	#[serde(default)]
	pub wifi_scan: bool,
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
