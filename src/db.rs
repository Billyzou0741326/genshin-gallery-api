use mongodb::bson::doc;
use mongodb::options::{ClientOptions, CreateCollectionOptions, IndexOptions};
use mongodb::{Client, Database, IndexModel};
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

pub async fn create_indexes(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<()>("artworks");
    collection
        .create_indexes(
            vec![
                IndexModel::builder()
                    .keys(doc! {
                        "art_id": 1,
                    })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
                IndexModel::builder()
                    .keys(doc! {
                        "upload_timestamp": 1,
                    })
                    .build(),
                IndexModel::builder()
                    .keys(doc! {
                        "characters": 1,
                    })
                    .build(),
                IndexModel::builder()
                    .keys(doc! {
                        "moderate.type": 1,
                    })
                    .build(),
                IndexModel::builder()
                    .keys(doc! {
                        "moderate.status": 1,
                    })
                    .build(),
            ],
            None,
        )
        .await?;
    Ok(())
}
