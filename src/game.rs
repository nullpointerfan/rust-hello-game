use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub character: Character,
    pub map: Map,
}

impl GameState {
    pub fn new(character: Character, map: Map) -> Self {
        Self { character, map }
    }

    pub fn move_character(&mut self, direction: &str) -> bool {
        let mut new_x = self.character.x;
        let mut new_y = self.character.y;

        match direction {
            "up" => new_y -= 1,
            "down" => new_y += 1,
            "left" => new_x -= 1,
            "right" => new_x += 1,
            _ => return false,
        }

        if self.map.is_walkable(new_x, new_y) {
            self.character.move_to(new_x, new_y);
            true
        } else {
            false
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