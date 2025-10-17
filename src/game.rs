use serde::{Deserialize, Serialize};
use actix::prelude::*;
use actix_web_actors::ws;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Character {
    pub x: i32,
    pub y: i32,
    pub health: i32,
}

impl Character {
    pub fn new(x: i32, y: i32, health: i32) -> Self {
        Self { x, y, health }
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<String>>,
}

impl Map {
    pub fn new(width: usize, height: usize, tiles: Vec<Vec<String>>) -> Self {
        Self { width, height, tiles }
    }

    pub fn is_valid_position(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        self.is_valid_position(x, y) && self.tiles[y as usize][x as usize] != "wall"
    }
}

#[derive(Clone)]
pub struct GameState {
    pub players: HashMap<String, Character>,
    pub map: Map,
    pub clients: Vec<actix::Addr<GameWebSocket>>,
}

impl GameState {
    pub fn new(map: Map) -> Self {
        Self { players: HashMap::new(), map, clients: Vec::new() }
    }

    pub fn add_player(&mut self, player_id: String) {
        if !self.players.contains_key(&player_id) {
            let character = Character::new(0, 0, 100);
            self.players.insert(player_id, character);
            self.notify_clients();
        }
    }

    pub fn move_character(&mut self, player_id: &str, direction: &str) -> bool {
        if let Some(character) = self.players.get_mut(player_id) {
            let mut new_x = character.x;
            let mut new_y = character.y;

            match direction {
                "up" => new_y -= 1,
                "down" => new_y += 1,
                "left" => new_x -= 1,
                "right" => new_x += 1,
                _ => return false,
            }

            if self.map.is_walkable(new_x, new_y) {
                character.move_to(new_x, new_y);
                self.notify_clients();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_character(&self, player_id: &str) -> Option<&Character> {
        self.players.get(player_id)
    }

    pub fn add_client(&mut self, addr: actix::Addr<GameWebSocket>) {
        self.clients.push(addr);
    }

    pub fn remove_client(&mut self, addr: &actix::Addr<GameWebSocket>) {
        self.clients.retain(|client| client != addr);
    }

    pub fn notify_clients(&self) {
        for client in &self.clients {
            let _ = client.do_send(UpdateGameState {
                players: self.players.clone(),
                map: self.map.clone(),
            });
        }
    }
}

pub fn create_default_map() -> Map {
    let wall_pattern = [
        [0,0,0,1,0,0,0,0,0,0],
        [0,1,0,1,0,1,0,0,0,0],
        [0,0,0,0,0,0,0,1,0,0],
        [1,0,1,0,0,0,0,0,0,0],
        [0,0,0,0,1,0,0,0,1,0],
        [0,0,0,0,0,0,1,0,0,0],
        [0,1,0,0,0,0,0,0,0,1],
        [0,0,0,1,0,0,0,1,0,0],
        [0,0,1,0,0,1,0,0,0,0],
        [0,0,0,0,0,0,0,0,1,0],
    ];

    let tiles = wall_pattern.iter().map(|row| {
        row.iter().map(|&cell| {
            if cell == 1 { "wall".to_string() } else { "empty".to_string() }
        }).collect()
    }).collect();

    Map::new(10, 10, tiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_new() {
        let character = Character::new(5, 10, 100);
        assert_eq!(character.x, 5);
        assert_eq!(character.y, 10);
        assert_eq!(character.health, 100);
    }

    #[test]
    fn test_character_move_to() {
        let mut character = Character::new(0, 0, 100);
        character.move_to(3, 7);
        assert_eq!(character.x, 3);
        assert_eq!(character.y, 7);
        assert_eq!(character.health, 100); // health should remain unchanged
    }

    #[test]
    fn test_map_new() {
        let tiles = vec![
            vec!["empty".to_string(), "wall".to_string()],
            vec!["wall".to_string(), "empty".to_string()],
        ];
        let map = Map::new(2, 2, tiles.clone());
        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.tiles, tiles);
    }

    #[test]
    fn test_map_is_valid_position() {
        let tiles = vec![
            vec!["empty".to_string(), "empty".to_string()],
            vec!["empty".to_string(), "empty".to_string()],
        ];
        let map = Map::new(2, 2, tiles);

        assert!(map.is_valid_position(0, 0));
        assert!(map.is_valid_position(1, 1));
        assert!(!map.is_valid_position(-1, 0));
        assert!(!map.is_valid_position(0, -1));
        assert!(!map.is_valid_position(2, 0));
        assert!(!map.is_valid_position(0, 2));
    }

    #[test]
    fn test_map_is_walkable() {
        let tiles = vec![
            vec!["empty".to_string(), "wall".to_string()],
            vec!["wall".to_string(), "empty".to_string()],
        ];
        let map = Map::new(2, 2, tiles);

        assert!(map.is_walkable(0, 0)); // empty
        assert!(!map.is_walkable(1, 0)); // wall
        assert!(!map.is_walkable(0, 1)); // wall
        assert!(map.is_walkable(1, 1)); // empty
        assert!(!map.is_walkable(-1, 0)); // out of bounds
        assert!(!map.is_walkable(0, -1)); // out of bounds
    }

    #[test]
    fn test_game_state_new() {
        let tiles = vec![vec!["empty".to_string()]];
        let map = Map::new(1, 1, tiles.clone());
        let game_state = GameState::new(map.clone());

        assert!(game_state.players.is_empty());
        assert_eq!(game_state.map.width, map.width);
        assert_eq!(game_state.map.height, map.height);
        assert!(game_state.clients.is_empty());
    }

    #[test]
    fn test_game_state_move_character_valid() {
        let tiles = vec![
            vec!["empty".to_string(), "empty".to_string()],
            vec!["empty".to_string(), "empty".to_string()],
        ];
        let map = Map::new(2, 2, tiles);
        let mut game_state = GameState::new(map);
        let player_id = "player1".to_string();
        game_state.add_player(player_id.clone());

        assert!(game_state.move_character(&player_id, "right"));
        if let Some(char) = game_state.players.get(&player_id) {
            assert_eq!(char.x, 1);
            assert_eq!(char.y, 0);
        }

        assert!(game_state.move_character(&player_id, "down"));
        if let Some(char) = game_state.players.get(&player_id) {
            assert_eq!(char.x, 1);
            assert_eq!(char.y, 1);
        }
    }

    #[test]
    fn test_game_state_move_character_invalid() {
        let tiles = vec![
            vec!["wall".to_string(), "wall".to_string()],
            vec!["empty".to_string(), "empty".to_string()],
        ];
        let map = Map::new(2, 2, tiles);
        let mut game_state = GameState::new(map);
        let player_id = "player1".to_string();
        game_state.add_player(player_id.clone());

        // Try to move into wall - should fail
        assert!(!game_state.move_character(&player_id, "right"));
        if let Some(char) = game_state.players.get(&player_id) {
            assert_eq!(char.x, 0);
            assert_eq!(char.y, 0);
        }

        // Try invalid direction - should fail
        assert!(!game_state.move_character(&player_id, "diagonal"));
        if let Some(char) = game_state.players.get(&player_id) {
            assert_eq!(char.x, 0);
            assert_eq!(char.y, 0);
        }

        // Try to move out of bounds - should fail
        assert!(!game_state.move_character(&player_id, "left"));
        if let Some(char) = game_state.players.get(&player_id) {
            assert_eq!(char.x, 0);
            assert_eq!(char.y, 0);
        }
    }

    #[test]
    fn test_create_default_map() {
        let map = create_default_map();
        assert_eq!(map.width, 10);
        assert_eq!(map.height, 10);
        assert_eq!(map.tiles.len(), 10);
        assert_eq!(map.tiles[0].len(), 10);

        // Check some known wall positions
        assert_eq!(map.tiles[0][3], "wall");
        assert_eq!(map.tiles[1][1], "wall");
        assert_eq!(map.tiles[0][0], "empty");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateGameState {
    pub players: HashMap<String, Character>,
    pub map: Map,
}

pub struct GameWebSocket {
    pub game_state: std::sync::Arc<std::sync::Mutex<GameState>>,
    pub player_id: Option<String>,
}

impl Actor for GameWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        if let Ok(mut game_state) = self.game_state.lock() {
            game_state.add_client(addr);
        }
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        if let Ok(mut game_state) = self.game_state.lock() {
            game_state.remove_client(&addr);
        }
    }
}

impl Handler<UpdateGameState> for GameWebSocket {
    type Result = ();

    fn handle(&mut self, msg: UpdateGameState, ctx: &mut Self::Context) {
        let data = serde_json::to_string(&serde_json::json!({
            "players": msg.players,
            "map": msg.map
        })).unwrap();
        ctx.text(data);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                if self.player_id.is_none() {
                    // First message should contain player ID
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(player_id) = data.get("playerId").and_then(|v| v.as_str()) {
                            self.player_id = Some(player_id.to_string());
                            if let Ok(mut game_state) = self.game_state.lock() {
                                game_state.add_player(player_id.to_string());
                            }
                        }
                    }
                }
                println!("Received: {}", text);
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}