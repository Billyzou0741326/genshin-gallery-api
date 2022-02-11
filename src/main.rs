use actix_web::web::Data;
use actix_web::{App, HttpServer};
use genshin_gallery_api::api::{
    api_all, api_character_ids, api_health, api_image_info, api_statistics,
};
use genshin_gallery_api::db::{create_client, create_indexes, create_views};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn_str = env::var("MONGODB_URI").expect("Environment variable MONGODB_URL is not set");
    let server_host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let server_port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_owned())
        .parse::<u16>()
        .unwrap_or(8000);
    let client = create_client(conn_str.as_str()).await.unwrap();
    let db = client.default_database().unwrap();
    create_indexes(&db).await.unwrap();
    create_views(&db).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .service(api_health)
            .service(api_statistics)
            .service(api_all)
            .service(api_character_ids)
            .service(api_image_info)
    })
    .bind(format!("{}:{}", server_host, server_port))?
    .run()
    .await
}
