use audiobookshelf_api::{ClientConfig, UserClient};
use dotenv;
use reqwest::Url;
use std::env::var;
use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let config = ClientConfig {
        root_url: Url::parse(&var("AUDIOBOOKSHELF_URL")?)?,
    };
    let username = var("AUDIOBOOKSHELF_USERNAME")?;
    let password = var("AUDIOBOOKSHELF_PASSWORD")?;

    println!("{username:?} {password:?}");
    let client = UserClient::auth(config, username, password).await?;
    let libraries = client.libraries().await?;
    println!("{:#?}", libraries);

    for library in &libraries {
        let library = client.library(&library.id).await?;
        println!("{:#?}", library);
    }

    Ok(())
}
