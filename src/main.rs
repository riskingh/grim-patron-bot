use std::error::Error;

mod client;
mod word_storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut ws = word_storage::WordStorage::default();
    ws.load_file("data/words_alpha.txt").ok();

    let mut client = client::create_client(ws).await?;
    client.start().await?;

    Ok(())
}
