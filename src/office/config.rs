use serde::{Deserialize, Serialize};

pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeConfig {
    pub version: u32,
    pub spawn: SpawnPoint,
    pub rooms: Vec<RoomConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub room: String,
    pub position: [f32; 3],
    pub yaw: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub id: String,
    pub name: String,
    pub origin: [f32; 3],
    pub size: [f32; 3],
    #[serde(default)]
    pub theme: RoomTheme,
    #[serde(default)]
    pub doors: Vec<DoorConfig>,
    #[serde(default)]
    pub desks: Vec<DeskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomTheme {
    pub wall_color: [f32; 3],
    pub floor_color: [f32; 3],
    pub ceiling_color: [f32; 3],
}

impl Default for RoomTheme {
    fn default() -> Self {
        Self {
            wall_color: [0.92, 0.88, 0.80],
            floor_color: [0.55, 0.40, 0.28],
            ceiling_color: [0.95, 0.95, 0.95],
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Wall {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorConfig {
    pub wall: Wall,
    pub offset: f32,
    pub width: f32,
    pub height: f32,
    pub to_room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeskConfig {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub position: [f32; 3],
    #[serde(default)]
    pub yaw: f32,
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
    #[serde(default = "default_monitor_size")]
    pub monitor_size: [f32; 2],
    #[serde(default)]
    pub startup: Option<StartupCommand>,
    #[serde(default)]
    pub character: Option<CharacterConfig>,
}

fn default_cols() -> u16 {
    100
}
fn default_rows() -> u16 {
    30
}
fn default_monitor_size() -> [f32; 2] {
    [1.4, 0.9]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StartupCommand {
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: Vec<(String, String)>,
    #[serde(default)]
    pub send: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConfig {
    #[serde(default = "default_char_color")]
    pub color: [f32; 3],
    #[serde(default)]
    pub shape: CharacterShape,
}

fn default_char_color() -> [f32; 3] {
    [0.7, 0.5, 0.9]
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum CharacterShape {
    #[default]
    Capsule,
    Cube,
    Sphere,
}

impl OfficeConfig {
    pub fn default_template() -> Self {
        let room = RoomConfig {
            id: "main".to_string(),
            name: "Main Office".to_string(),
            origin: [0.0, 0.0, 0.0],
            size: [10.0, 3.0, 10.0],
            theme: RoomTheme::default(),
            doors: Vec::new(),
            desks: vec![DeskConfig {
                id: "desk-1".to_string(),
                name: Some("Clanker #1".to_string()),
                position: [0.0, 0.0, -4.5],
                yaw: 0.0,
                cols: 100,
                rows: 30,
                monitor_size: [1.8, 1.0],
                startup: None,
                character: Some(CharacterConfig {
                    color: default_char_color(),
                    shape: CharacterShape::Capsule,
                }),
            }],
        };
        Self {
            version: CURRENT_VERSION,
            spawn: SpawnPoint {
                room: "main".to_string(),
                position: [0.0, 1.65, 1.0],
                yaw: 0.0,
            },
            rooms: vec![room],
        }
    }

    pub fn room(&self, id: &str) -> Option<&RoomConfig> {
        self.rooms.iter().find(|r| r.id == id)
    }

    pub fn room_mut(&mut self, id: &str) -> Option<&mut RoomConfig> {
        self.rooms.iter_mut().find(|r| r.id == id)
    }
}
