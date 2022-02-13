use crate::artwork::ArtworkInfo;
use actix_web::web::{Data, Query};
use actix_web::{get, http, post, web, HttpRequest, HttpResponse, Responder};
use mongodb::Database;
use serde::Deserialize;
use serde_json::json;
use serde_qs;
use std::str::from_utf8;
use tokio::join;

use crate::db::{
    get_artwork_count_nsfw, get_artwork_count_r18, get_artwork_count_sfw, get_artwork_count_total,
    get_artwork_info_by_ids, get_ids, get_latest_upload_time, save_artwork_many,
    ArtworkQueryOption,
};

pub struct DbSyncToken(String);

impl DbSyncToken {
    pub fn new(token: String) -> Self {
        DbSyncToken { 0: token }
    }

    pub fn token(&self) -> String {
        self.0.clone()
    }
}

#[derive(Deserialize)]
pub struct ArtworkIdRequest {
    #[serde(rename = "type")]
    art_type: Option<String>,
    character: Option<String>,
}

#[derive(Deserialize)]
pub struct ArtworkInfoRequest {
    ids: Vec<i64>,
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

#[get("/api/image-info")]
pub async fn api_image_info(db: Data<Database>, req: HttpRequest) -> impl Responder {
    // Need to explicitly parse the query string since they're arrays
    // https://github.com/samscott89/serde_qs/blob/main/examples/introduction.rs
    let query = req.query_string();
    match serde_qs::from_str::<ArtworkInfoRequest>(query) {
        Ok(info) => match get_artwork_info_by_ids(&db, info.ids).await {
            Ok(artwork_info) => HttpResponse::Ok()
                .content_type("application/json")
                .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
                .body(
                    json!({
                        "data": artwork_info,
                    })
                    .to_string(),
                ),
            Err(_) => HttpResponse::InternalServerError()
                .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
                .body(""),
        },
        Err(_) => HttpResponse::BadRequest()
            .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
            .body(""),
    }
}

#[get("/api/statistics")]
pub async fn api_statistics(db: Data<Database>) -> impl Responder {
    let results = join! {
        get_artwork_count_total(&db),
        get_latest_upload_time(&db),
        get_artwork_count_sfw(&db),
        get_artwork_count_nsfw(&db),
        get_artwork_count_r18(&db),
    };
    let total = results.0.unwrap_or(0);
    let days = results.1.unwrap_or(0);
    let sfw = results.2.unwrap_or(0);
    let nsfw = results.3.unwrap_or(0);
    let r18 = results.4.unwrap_or(0);
    let body = json! ({
        "total": total,
        "sfw": sfw,
        "nsfw": nsfw,
        "r18": r18,
        "latestUploadDays": days,
    });
    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
        .body(body.to_string())
}

#[post("/api/db/sync")]
pub async fn api_db_sync(
    db: Data<Database>,
    db_sync_token: Data<DbSyncToken>,
    web::Json(artwork_list): web::Json<Vec<ArtworkInfo>>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(err) = validate_db_sync_token(db_sync_token.token(), req.headers()) {
        return HttpResponse::BadRequest()
            .content_type("application/json")
            .body(
                json!({
                    "message": err.to_string(),
                })
                .to_string(),
            );
    }
    match save_artwork_many(&db, artwork_list).await {
        Ok(()) => HttpResponse::Ok().content_type("application/json").body(
            json!({
                "message": "ok",
            })
            .to_string(),
        ),
        Err(_) => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(
                json! ({
                    "message": "error",
                })
                .to_string(),
            ),
    }
}

fn validate_db_sync_token(
    db_sync_token: String,
    headers: &http::header::HeaderMap,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(authorization_header) = headers.get(http::header::AUTHORIZATION) {
        if let Ok(authorization_str) = from_utf8(authorization_header.as_bytes()) {
            let expected = format!("Bearer {}", db_sync_token);
            if authorization_str == expected {
                return Ok(());
            }
            return Err("Invalid token".into());
        }
        return Err("Invalid authorization header format".into());
    }
    Err("Authorization header not found".into())
}
