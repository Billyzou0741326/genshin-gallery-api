use mongodb::bson::doc;
use mongodb::options::{ClientOptions, CreateCollectionOptions};
use mongodb::{Client, Database};
use tokio::join;

pub async fn create_client(conn_str: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let client_options = ClientOptions::parse(conn_str).await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}

pub async fn create_views(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let collection_name = "artworks";
    let _ = join! {
        db.create_collection(
            "artworks_sfw",
            CreateCollectionOptions::builder()
                .view_on(collection_name.to_owned())
                .pipeline(vec! { doc! {
                    "$match": {
                        "is_404": { "$ne": true },
                        "moderate.type": "SFW",
                        "$or":[
                            { "moderate.status": "PASS" },
                            { "moderate.status": "PUSH" },
                        ],
                    },
                }})
                .build(),
        ),
        db.create_collection(
            "artworks_nsfw",
            CreateCollectionOptions::builder()
                .view_on(collection_name.to_owned())
                .pipeline(vec! { doc! {
                    "$match": {
                        "is_404": { "$ne": true },
                        "moderate.type": "NSFW",
                        "$or":[
                            { "moderate.status": "PASS" },
                            { "moderate.status": "PUSH" },
                        ],
                    },
                }})
                .build(),
        ),
        db.create_collection(
            "artworks_r18",
            CreateCollectionOptions::builder()
                .view_on(collection_name.to_owned())
                .pipeline(vec! { doc! {
                    "$match": {
                        "is_404": { "$ne": true },
                        "moderate.type": "R18",
                        "$or":[
                            { "moderate.status": "PASS" },
                            { "moderate.status": "PUSH" },
                        ],
                    },
                }})
                .build(),
        ),
    };
    Ok(())
}
