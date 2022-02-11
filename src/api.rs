use actix_web::web::{Data, Query};
use actix_web::{get, http, web, HttpResponse, Responder};
use mongodb::Database;
use serde::Deserialize;
use serde_json::json;

use crate::db::{get_ids, ArtworkQueryOption};

#[derive(Deserialize)]
pub struct ArtworkIdRequest {
    #[serde(rename = "type")]
    art_type: Option<String>,
    character: Option<String>,
}

#[get("/api/health")]
pub async fn api_health() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body("{}")
}

#[get("/api/all")]
pub async fn api_all(db: Data<Database>, Query(info): Query<ArtworkIdRequest>) -> impl Responder {
    let characters = match info.character {
        Some(character) => vec![character],
        None => vec![],
    };
    let art_type = info.art_type.unwrap_or_else(|| "SFW".to_owned());
    let options = ArtworkQueryOption::builder()
        .characters(characters)
        .image_type(art_type)
        .build();
    let get_id_result = get_ids(&db, options).await;
    match get_id_result {
        Ok(id_list) => HttpResponse::Ok()
            .content_type("application/json")
            .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .body(
                json! ({
                    "data": id_list,
                })
                .to_string(),
            ),
        Err(e) => HttpResponse::InternalServerError()
            .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .body(e.to_string()),
    }
}

#[get("/api/character/{name}")]
pub async fn api_character_ids(
    db: Data<Database>,
    params: web::Path<(String,)>,
    Query(info): Query<ArtworkIdRequest>,
) -> impl Responder {
    let (name,) = params.into_inner();
    let art_type = info.art_type.unwrap_or_else(|| "SFW".to_owned());
    let options = ArtworkQueryOption::builder()
        .characters(vec![name])
        .image_type(art_type)
        .build();
    match get_ids(&db, options).await {
        Ok(id_list) => HttpResponse::Ok()
            .content_type("application/json")
            .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .body(
                json! ({
                    "data": id_list,
                })
                .to_string(),
            ),
        Err(e) => HttpResponse::InternalServerError()
            .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .body(e.to_string()),
    }
}
