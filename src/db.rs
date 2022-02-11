use crate::artwork::ArtworkInfo;
use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, CreateCollectionOptions, IndexOptions};
use mongodb::{bson, Client, Database, IndexModel};
use serde::{Deserialize, Serialize};
use tokio::join;
use tokio_stream::StreamExt;
use typed_builder::TypedBuilder;

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

#[derive(Clone, Debug, Deserialize, TypedBuilder, Serialize)]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct ArtworkQueryOption {
    pub characters: Option<Vec<String>>,
    pub image_type: Option<String>,
}

fn filter_conditions(options: &ArtworkQueryOption) -> Vec<Document> {
    return match &options.characters {
        Some(characters) => {
            let character_filters: Vec<Document> = characters
                .iter()
                .filter(|chara| !chara.trim().is_empty())
                .map(|chara| {
                    doc! {
                        "characters": {
                            "$regex": chara.trim(),
                            "$options": "i",
                        }
                    }
                })
                .collect();
            return character_filters;
        }
        None => vec![],
    };
}

fn collection_name_by_artwork_type(artwork_type: String) -> String {
    let t = artwork_type.to_uppercase();
    if t == "SFW" {
        return "artworks_sfw".to_owned();
    } else if t == "NSFW" {
        return "artworks_nsfw".to_owned();
    } else if t == "R18" {
        return "artworks_r18".to_owned();
    }
    "artworks_sfw".to_owned()
}

pub async fn get_ids(
    db: &Database,
    options: impl Into<Option<ArtworkQueryOption>>,
) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let mut filtering = doc! { "$match": {} };
    let filtering_match = filtering.get_document_mut("$match").unwrap();
    let mut filtering_match_conditions = vec![];
    let mut collection_name = "artworks_sfw".to_owned();
    if let Some(val) = options.into() {
        filtering_match_conditions = filter_conditions(&val);
        if let Some(artwork_type) = &val.image_type {
            collection_name = collection_name_by_artwork_type(artwork_type.clone());
        }
    }
    if !filtering_match_conditions.is_empty() {
        filtering_match.insert("$or", filtering_match_conditions);
    }
    let collection = db.collection::<ArtworkInfo>(&collection_name);
    let query_aggregate = vec![
        filtering,
        doc! { "$sort": { "upload_timestamp": -1 } },
        doc! { "$project": { "art_id": 1 } },
    ];
    let cursor = collection.aggregate(query_aggregate, None).await?;
    let result = cursor
        .map(|item| match item {
            Ok(val) => {
                if let Ok(art_id) = val.get_i32("art_id") {
                    return art_id.into();
                }
                val.get_i64("art_id").unwrap_or(-1)
            }
            Err(_) => -1,
        })
        .filter(|val| *val != -1)
        .collect()
        .await;
    Ok(result)
}

pub async fn get_artwork_info_by_ids(
    db: &Database,
    id_list: Vec<i64>,
) -> Result<Vec<ArtworkInfo>, Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks");
    let filtering_match_conditions: Vec<Document> = id_list
        .iter()
        .map(|art_id| doc! { "art_id": art_id })
        .collect();
    let filtering = doc! {
        "$match": { "$or": filtering_match_conditions },
    };
    let pipeline = vec![filtering];
    let mut result: Vec<ArtworkInfo> = vec![];
    let mut cursor = collection.aggregate(pipeline, None).await?;
    while let Some(val) = cursor.next().await {
        if let Ok(document) = val {
            if let Ok(artwork) = bson::from_document(document) {
                result.push(artwork);
            }
        }
    }
    Ok(result)
}

