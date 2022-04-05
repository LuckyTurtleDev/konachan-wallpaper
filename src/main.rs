mod konachan;
use konachan::*;

#[tokio::main]
async fn main() {
	get_post(&vec!["rating:safe".to_string()], 1100).await;
}
