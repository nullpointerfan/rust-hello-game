use actix_web::{web, Result, HttpResponse};
use actix_web_actors::ws;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use crate::game::{GameState, GameWebSocket};

pub type AppState = Arc<Mutex<GameState>>;

pub async fn hello() -> Result<String> {
    Ok("Hello world!".to_string())
}

pub async fn get_character(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<impl actix_web::Responder> {
    let game_state = data.lock().map_err(|_| actix_web::error::ErrorInternalServerError("Lock failed"))?;
    let player_id = req.headers()
        .get("x-player-id")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing x-player-id header"))?;

    if let Some(character) = game_state.get_character(player_id) {
        Ok(web::Json(character.clone()))
    } else {
        Err(actix_web::error::ErrorNotFound("Player not found"))
    }
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
    http_req: actix_web::HttpRequest,
) -> Result<impl actix_web::Responder> {
    let mut game_state = data.lock().map_err(|_| actix_web::error::ErrorInternalServerError("Lock failed"))?;
    let player_id = http_req.headers()
        .get("x-player-id")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing x-player-id header"))?;

    if game_state.move_character(player_id, &req.direction) {
        if let Some(character) = game_state.get_character(player_id) {
            Ok(web::Json(character.clone()))
        } else {
            Err(actix_web::error::ErrorNotFound("Player not found"))
        }
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

    ws::start(GameWebSocket { game_state, player_id: None }, &req, stream)
}