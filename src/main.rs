mod game;
mod web;

use actix_web::{App, HttpServer};
use actix_web::web::{Data, get, post};
use std::sync::Arc;
use crate::game::{GameState, create_default_map};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let map = create_default_map();
    let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));

    let app_data = Data::new(game_state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/", get().to(crate::web::hello))
            .route("/game", get().to(crate::web::game_page))
            .route("/character", get().to(crate::web::get_character))
            .route("/map", get().to(crate::web::get_map))
            .route("/move", post().to(crate::web::move_character))
            .route("/ws", get().to(crate::web::websocket))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
