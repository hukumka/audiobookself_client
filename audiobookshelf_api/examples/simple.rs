use audiobookshelf_api::params::{LibraryItemFilter, LibraryItemParams};
use audiobookshelf_api::{ClientConfig, UserClient};
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
    let library = client.libraries().await?.pop().unwrap();
    println!("{:#?}", library);

    let filters = client.library(&library.id).await?.filterdata;
    let items = client
        .library_items(
            &library.id,
            LibraryItemParams {
                filter: LibraryItemFilter {
                    series: vec![filters.series[0].id.clone()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await?;

    println!("{:#?}", items);

    let item = client.library_item(&items[0].id).await?;
    println!("{:#?}", item);

    Ok(())
}
