#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use actix_rt;
    use std::sync::Arc;
    use hello_cargo::game::{GameState, Character, create_default_map};
    use hello_cargo::web::{hello, get_character, get_map, move_character};

    #[actix_rt::test]
    async fn test_hello() {
        let app = test::init_service(App::new().route("/", web::get().to(hello))).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert_eq!(body, "Hello world!");
    }

    #[actix_rt::test]
    async fn test_get_character() {
        let character = Character::new(5, 10, 100);
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(character, map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/character", web::get().to(get_character))
        ).await;

        let req = test::TestRequest::get().uri("/character").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["x"], 5);
        assert_eq!(body["y"], 10);
        assert_eq!(body["health"], 100);
    }

    #[actix_rt::test]
    async fn test_get_map() {
        let character = Character::new(0, 0, 100);
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(character, map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/map", web::get().to(get_map))
        ).await;

        let req = test::TestRequest::get().uri("/map").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["width"], 10);
        assert_eq!(body["height"], 10);
        assert!(body["tiles"].is_array());
    }

    #[actix_rt::test]
    async fn test_move_character_valid() {
        let character = Character::new(0, 0, 100);
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(character, map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/move", web::post().to(move_character))
        ).await;

        let req = test::TestRequest::post()
            .uri("/move")
            .set_json(&serde_json::json!({"direction": "down"}))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["x"], 0);
        assert_eq!(body["y"], 1);
        assert_eq!(body["health"], 100);
    }

    #[actix_rt::test]
    async fn test_move_character_invalid() {
        let character = Character::new(0, 0, 100);
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(character, map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/move", web::post().to(move_character))
        ).await;

        let req = test::TestRequest::post()
            .uri("/move")
            .set_json(&serde_json::json!({"direction": "invalid"}))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
        let body = test::read_body(resp).await;
        assert_eq!(body, "Invalid move");
    }

    #[actix_rt::test]
    async fn test_websocket_route_exists() {
        let character = Character::new(0, 0, 100);
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(character, map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/ws", web::get().to(hello_cargo::web::websocket))
        ).await;

        // WebSocket handshake requires specific headers, so we just check that the route exists
        // by making a regular GET request which should fail with 400 (no upgrade header)
        let req = test::TestRequest::get().uri("/ws").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
    }
}