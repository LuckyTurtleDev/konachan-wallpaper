mod konachan;
use konachan::*;

use once_cell::sync::Lazy;
use reqwest::Client;

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

#[tokio::main]
async fn main() {
	get_posts(&vec!["rating:safe".to_string()], 100).await;
}
