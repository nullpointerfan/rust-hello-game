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
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
        {
            let mut gs = game_state.lock().unwrap();
            gs.add_player("test_player".to_string());
        }
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/character", web::get().to(get_character))
        ).await;

        let req = test::TestRequest::get()
            .uri("/character")
            .insert_header(("x-player-id", "test_player"))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["x"], 0);
        assert_eq!(body["y"], 0);
        assert_eq!(body["health"], 100);
    }

    #[actix_rt::test]
    async fn test_get_character_missing_header() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/character", web::get().to(get_character))
        ).await;

        let req = test::TestRequest::get()
            .uri("/character")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_get_map() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
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
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
        {
            let mut gs = game_state.lock().unwrap();
            gs.add_player("test_player".to_string());
        }
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/move", web::post().to(move_character))
        ).await;

        let req = test::TestRequest::post()
            .uri("/move")
            .insert_header(("x-player-id", "test_player"))
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
    async fn test_move_character_missing_header() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
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

        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_move_character_invalid() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
        {
            let mut gs = game_state.lock().unwrap();
            gs.add_player("test_player".to_string());
        }
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/move", web::post().to(move_character))
        ).await;

        let req = test::TestRequest::post()
            .uri("/move")
            .insert_header(("x-player-id", "test_player"))
            .set_json(&serde_json::json!({"direction": "invalid"}))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
        let body = test::read_body(resp).await;
        assert_eq!(body, "Invalid move");
    }

    #[actix_rt::test]
    async fn test_websocket_route_exists() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
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

    #[actix_rt::test]
    async fn test_websocket_missing_header() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/ws", web::get().to(hello_cargo::web::websocket))
        ).await;

        let req = test::TestRequest::get().uri("/ws").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
    }

    #[actix_rt::test]
    async fn test_multiple_players() {
        let map = create_default_map();
        let game_state = Arc::new(std::sync::Mutex::new(GameState::new(map)));
        {
            let mut gs = game_state.lock().unwrap();
            gs.add_player("player1".to_string());
            gs.add_player("player2".to_string());
        }
        let app_data = web::Data::new(game_state);

        let app = test::init_service(
            App::new()
                .app_data(app_data)
                .route("/character", web::get().to(get_character))
        ).await;

        // Test player1
        let req1 = test::TestRequest::get()
            .uri("/character")
            .insert_header(("x-player-id", "player1"))
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        assert!(resp1.status().is_success());

        // Test player2
        let req2 = test::TestRequest::get()
            .uri("/character")
            .insert_header(("x-player-id", "player2"))
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        assert!(resp2.status().is_success());

        // Test non-existent player
        let req3 = test::TestRequest::get()
            .uri("/character")
            .insert_header(("x-player-id", "player3"))
            .to_request();
        let resp3 = test::call_service(&app, req3).await;
        assert_eq!(resp3.status(), 404);
    }
}