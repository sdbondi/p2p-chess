use std::{fmt, marker::PhantomData};

use pleco::Player;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
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
}

fn serialize_player<S: Serializer>(player: &Player, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_u8(*player as u8)
}

fn deserialize_player<'de, D>(des: D) -> Result<Player, D::Error>
where D: Deserializer<'de> {
    struct Visitor<K> {
        marker: PhantomData<K>,
    }

    impl<'de> de::Visitor<'de> for Visitor<Player> {
        type Value = Player;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a u8 repr player")
        }

        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
        where E: de::Error {
            match v {
                0 => Ok(Player::Black),
                1 => Ok(Player::White),
                _ => Err(E::custom("invalid player byte")),
            }
        }
    }
    des.deserialize_u8(Visitor { marker: PhantomData })
}
