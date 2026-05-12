use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Exploring,
    Typing(Entity),
}

impl GameState {
    pub fn is_typing(&self) -> bool {
        matches!(self, GameState::Typing(_))
    }

    pub fn focused_entity(&self) -> Option<Entity> {
        match self {
            GameState::Typing(e) => Some(*e),
            _ => None,
        }
    }
}

#[derive(Component)]
pub struct TerminalFocus;
