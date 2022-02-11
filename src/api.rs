use actix_web::web::{Data, Query};
use actix_web::{get, http, web, HttpRequest, HttpResponse, Responder};
use mongodb::Database;
use serde::Deserialize;
use serde_json::json;
use serde_qs;
use tokio::join;

use crate::db::{
    get_artwork_count_nsfw, get_artwork_count_r18, get_artwork_count_sfw, get_artwork_count_total,
    get_artwork_info_by_ids, get_ids, get_latest_upload_time, ArtworkQueryOption,
};

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
