use std::ops::Index;

use pleco::Player;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use tari_comms::types::CommsPublicKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: u32,
    pub opponent: CommsPublicKey,
    pub board_fen: String,
    pub seq: u32,
    #[serde(serialize_with = "serialize_player", deserialize_with = "deserialize_player")]
    pub player: Player,
    pub result: GameResult,
    pub last_activity: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GameResult {
    None,
    Checkmate(#[serde(serialize_with = "serialize_player", deserialize_with = "deserialize_player")] Player),
    Draw,
    WeResigned,
    TheyResigned,
}

impl Game {
    pub fn has_completed(&self) -> bool {
        !matches!(self.result, GameResult::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCollection {
    games: Vec<Game>,
}

impl GameCollection {
    pub fn get_mut(&mut self, game_id: u32) -> Option<&mut Game> {
        self.games.iter_mut().find(|g| g.id == game_id)
    }

    /// Returns true if another game for same opponent already exists
    pub fn insert(&mut self, game: Game) -> bool {
        match self.get_mut(game.id) {
            Some(game_mut) => {
                *game_mut = game;
                true
            },
            None => {
                self.games.push(game);
                false
            },
        }
    }

    pub fn clean_up(&mut self) {
        self.games = self
            .games
            .drain(..)
            .enumerate()
            // Keep last 10, after that only keep active games
            .filter(|(i, g)| *i < 10 || !g.has_completed())
            .map(|(_, g)| g)
            .collect();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Game> {
        self.games.iter()
    }

    pub fn sort(&mut self) {
        self.games.sort_by(|a, b| a.last_activity.cmp(&b.last_activity));
    }
}

impl Index<usize> for GameCollection {
    type Output = Game;

    fn index(&self, index: usize) -> &Self::Output {
        self.games.index(index)
    }
}

fn serialize_player<S: Serializer>(player: &Player, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_u8(*player as u8)
}

fn deserialize_player<'de, D>(des: D) -> Result<Player, D::Error>
where D: Deserializer<'de> {
    match <u8 as serde::Deserialize>::deserialize(des) {
        Ok(0) => Ok(Player::White),
        Ok(1) => Ok(Player::Black),
        _ => Err(D::Error::custom("invalid player byte")),
    }
}
