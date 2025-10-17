use actix_web::{web, Result, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use crate::game::{GameState, GameWebSocket};

pub type AppState = Arc<Mutex<GameState>>;

pub async fn hello() -> Result<String> {
    Ok("Hello world!".to_string())
}

pub async fn get_character(data: web::Data<AppState>) -> Result<impl actix_web::Responder> {
    let game_state = data.lock().map_err(|_| actix_web::error::ErrorInternalServerError("Lock failed"))?;
    Ok(web::Json(game_state.character.clone()))
}

pub async fn get_map(data: web::Data<AppState>) -> Result<impl actix_web::Responder> {
    let game_state = data.lock().map_err(|_| actix_web::error::ErrorInternalServerError("Lock failed"))?;
    Ok(web::Json(game_state.map.clone()))
}

#[derive(Deserialize)]
pub struct MoveRequest {
    pub direction: String,
}

pub async fn move_character(
    data: web::Data<AppState>,
    req: web::Json<MoveRequest>,
) -> Result<impl actix_web::Responder> {
    let mut game_state = data.lock().map_err(|_| actix_web::error::ErrorInternalServerError("Lock failed"))?;

    if game_state.move_character(&req.direction) {
        Ok(web::Json(game_state.character.clone()))
    } else {
        Err(actix_web::error::ErrorBadRequest("Invalid move"))
    }
}

pub async fn game_page() -> Result<impl actix_web::Responder> {
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/game.html")))
}

pub async fn websocket(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<impl actix_web::Responder> {
    let game_state = data.get_ref().clone();
    ws::start(GameWebSocket { game_state }, &req, stream)
}