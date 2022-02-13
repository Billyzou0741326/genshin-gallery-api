use crate::artwork::ArtworkInfo;
use futures::future::join_all;
use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, CreateCollectionOptions, IndexOptions, ReplaceOptions};
use mongodb::{bson, Client, Database, IndexModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::join;
use tokio_stream::StreamExt;
use typed_builder::TypedBuilder;

/// Creates a mongodb client object from a connection string.
/// The connection string is preferably provided at runtime via environment variable
pub async fn create_client(conn_str: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let client_options = ClientOptions::parse(conn_str).await?;
    let client = Client::with_options(client_options)?;
    Ok(client)
}

/// Create views to simplify queries
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

/// Create indexes to enforce constraints and speed up queries
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

/// Parses the common query options
#[derive(Clone, Debug, Deserialize, TypedBuilder, Serialize)]
#[builder(field_defaults(default, setter(strip_option)))]
pub struct ArtworkQueryOption {
    pub characters: Option<Vec<String>>,
    pub image_type: Option<String>,
}

/// Condition applied to database queries
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

fn collection_name_by_artwork_type(artwork_type: &str) -> &str {
    match artwork_type.to_string().to_uppercase().as_str() {
        "SFW" => "artworks_sfw",
        "NSFW" => "artworks_nsfw",
        "R18" => "artworks_r18",
        _ => "artworks_sfw",
    }
}

/// Get artwork ids
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
            collection_name = collection_name_by_artwork_type(artwork_type.as_str())
                .parse()
                .unwrap();
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

/// Get artwork info (metadata), e.g title, tags, url
pub async fn get_artwork_info_by_ids(
    db: &Database,
    id_list: Vec<i64>,
) -> Result<Vec<ArtworkInfo>, Box<dyn std::error::Error>> {
    if id_list.is_empty() {
        return Ok(vec![]);
    }
    let collection = db.collection::<ArtworkInfo>("artworks");
    let filtering_match_conditions: Vec<Document> = id_list
        .iter()
        .map(|art_id| doc! { "art_id": art_id })
        .collect();
    let filtering = doc! { "$match": { "$or": filtering_match_conditions } };
    let pipeline = vec![filtering];
    let mut map: HashMap<i64, ArtworkInfo> = HashMap::with_capacity(id_list.len());
    let mut result: Vec<ArtworkInfo> = vec![];
    let mut cursor = collection.aggregate(pipeline, None).await?;
    while let Some(cursor_result) = cursor.next().await {
        if let Ok(document) = cursor_result {
            if let Ok(artwork) = bson::from_document(document) {
                let art: ArtworkInfo = artwork;
                map.insert(art.art_id, art);
            }
        }
    }
    for art_id in id_list {
        if let Some(artwork) = map.get(&art_id) {
            result.push(artwork.clone());
        }
    }
    Ok(result)
}

/// Get upload time of the most recent upload
pub async fn get_latest_upload_time(db: &Database) -> Result<i64, Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks");
    let pipeline = vec![
        doc! { "$project": { "art_id": 1, "upload_timestamp": 1 } },
        doc! { "$sort": { "upload_timestamp": -1, "art_id": 1 } },
        doc! { "$limit": 1 },
    ];
    let mut cursor = collection.aggregate(pipeline, None).await?;
    while let Some(val) = cursor.next().await {
        if let Ok(document) = val {
            if let Ok(upload_timestamp) = document.get_i32("upload_timestamp") {
                return Ok(upload_timestamp.into());
            }
            if let Ok(upload_timestamp) = document.get_i64("upload_timestamp") {
                return Ok(upload_timestamp);
            }
        }
    }
    Ok(0)
}

/// Get total artwork stored in the database
pub async fn get_artwork_count_total(db: &Database) -> Result<u64, Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks");
    let result = collection.count_documents(None, None).await?;
    Ok(result)
}

/// Get sfw artwork count
pub async fn get_artwork_count_sfw(db: &Database) -> Result<u64, Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks_sfw");
    let result = collection.count_documents(None, None).await?;
    Ok(result)
}

/// Get nsfw artwork count
pub async fn get_artwork_count_nsfw(db: &Database) -> Result<u64, Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks_nsfw");
    let result = collection.count_documents(None, None).await?;
    Ok(result)
}

/// Get r18 artwork count
pub async fn get_artwork_count_r18(db: &Database) -> Result<u64, Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks_r18");
    let result = collection.count_documents(None, None).await?;
    Ok(result)
}

/// Update database artwork
/// TODO: waiting for mongodb rust driver to implement bulk write support
pub async fn save_artwork_many(
    db: &Database,
    artwork_list: Vec<ArtworkInfo>,
) -> Result<(), Box<dyn std::error::Error + '_>> {
    let join_handles = artwork_list
        .iter()
        .map(|artwork| save_artwork_one(db, artwork.clone()));
    let _result = join_all(join_handles).await;
    Ok(())
}

/// use `replace_one` to upsert an artwork entry
pub async fn save_artwork_one(
    db: &Database,
    artwork: ArtworkInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<ArtworkInfo>("artworks");
    match collection
        .replace_one(
            doc! { "art_id": artwork.art_id },
            artwork.clone(),
            ReplaceOptions::builder().upsert(true).build(),
        )
        .await
    {
        Ok(_) => Ok(()),
        _ => Err(format!("failed to save artwork {:?}", artwork).into()),
    }
}
