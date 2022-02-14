use actix_web::web::Data;
use actix_web::{App, HttpServer};
use genshin_gallery_api::api::{
    api_all, api_character_ids, api_db_sync, api_health, api_image_info, api_statistics,
    DbSyncToken,
};
use genshin_gallery_api::db::{create_client, create_indexes, create_views};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Read environment variables
    let conn_str = env::var("MONGODB_URL").expect("Environment variable MONGODB_URL is not set");
    let server_host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());
    let server_port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_owned())
        .parse::<u16>()
        .unwrap_or(8000);
    let db_sync_token = env::var("DB_SYNC_TOKEN").unwrap_or_default();  // not safe

    // Connect to mongodb
    let client = create_client(conn_str.as_str()).await.unwrap();
    let db = client.database("pixiv");
    if let Err(e) = create_indexes(&db).await {
        println!("{:?}", e)
    }
    if let Err(e) = create_views(&db).await {
        println!("{:?}", e)
    }

    // Launch http webserver
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(DbSyncToken::new(db_sync_token.to_owned())))
            .service(api_health)
            .service(api_statistics)
            .service(api_all)
            .service(api_character_ids)
            .service(api_image_info)
            .service(api_db_sync)
    })
    .bind(format!("{}:{}", server_host, server_port))?
    .run()
    .await
}
