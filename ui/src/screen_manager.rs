use std::{fs, path::Path};

use anyhow::anyhow;
use minifb::Window;
use p2p_chess_channel::{ChessOperation, MessageChannel, OperationType, TryRecvError};
use pleco::{BitMove, Player};
use rand::{rngs::OsRng, RngCore};
use tari_comms::types::CommsPublicKey;
use tari_utilities::hex::Hex;

use crate::{
    board,
    clipboard::Clipboard,
    color::Color,
    drawable::{Drawable, FrameBuffer},
    game::{Game, GameCollection, GameResult},
    game_screen::{GameConfig, GameScreen, GameStatus},
    start_screen::StartScreen,
};

#[derive(Debug)]
pub struct ScreenManager {
    config: GameConfig,
    active_screen: Screen,
    clipboard: Clipboard,
    public_key: CommsPublicKey,
    channel: MessageChannel<ChessOperation>,
    games: GameCollection,
}

impl ScreenManager {
    pub fn initialize(
        config: GameConfig,
        channel: MessageChannel<ChessOperation>,
        public_key: CommsPublicKey,
    ) -> anyhow::Result<Self> {
        let clipboard = Clipboard::initialize()?;
        let games = load_games(&config.save_path)?;
        Ok(Self {
            config,
            active_screen: Screen::Start(StartScreen::new(clipboard.clone(), public_key.clone())),
            public_key,
            clipboard,
            channel,
            games,
        })
    }

    fn create_new_game(&mut self, opponent: CommsPublicKey) {
        // TODO: allow player to choose black/white
        self.active_screen = Screen::Game(GameScreen::new(
            self.config.clone(),
            Player::White,
            opponent.clone(),
            None,
        ));
        let id = OsRng.next_u32();
        self.games.insert(Game {
            id,
            opponent: opponent.clone(),
            board_fen: board::INITIAL_BOARD.to_string(),
            seq: 0,
            player: Player::White,
            result: GameResult::None,
        });
        self.channel
            .try_send(ChessOperation {
                game_id: id,
                seq: 0,
                opponent,
                operation: OperationType::NewGame {
                    player: Player::Black as u8,
                },
            })
            .unwrap();
    }

    pub fn render(&mut self, window: &Window, buf: &mut FrameBuffer) {
        match self.active_screen {
            Screen::Start(ref mut main_screen) => {
                main_screen.update(window);
                main_screen.set_games(&self.games);
                main_screen.draw(buf);
                let idx = main_screen.show_game_clicked();
                if let Some(pk) = main_screen.new_game_clicked() {
                    match CommsPublicKey::from_hex(pk) {
                        Ok(pk) => {
                            buf.clear(Color::black());
                            self.create_new_game(pk);
                        },
                        Err(_) => {
                            main_screen.set_input_error("Invalid public key");
                        },
                    }
                }
                if let Some(idx) = idx {
                    dbg!(idx);
                    let game = &self.games[idx];
                    buf.clear(Color::black());
                    self.active_screen = Screen::Game(GameScreen::new(
                        self.config.clone(),
                        game.player,
                        game.opponent.clone(),
                        Some(game.board_fen.clone()),
                    ));
                }
            },
            Screen::Game(ref mut game) => {
                game.draw(buf);
                game.update(&window);
                match game.state().game_status() {
                    // TODO: display winner
                    GameStatus::StaleMate | GameStatus::CheckMate(_) | GameStatus::Resign(_) => {
                        buf.clear(Color::black());
                        self.active_screen =
                            Screen::Start(StartScreen::new(self.clipboard.clone(), self.public_key.clone()));
                    },
                    _ => {},
                }
            },
        }

        match self.channel.try_recv() {
            Ok(op) => {
                if let Err(err) = self.apply_operation(op) {
                    log::error!("apply operation failed: {}", err);
                }
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {
                // TODO
            },
        }
    }

    pub fn apply_operation(&mut self, op: ChessOperation) -> anyhow::Result<()> {
        match &op.operation {
            OperationType::NewGame { player } => {
                let game = Game {
                    id: op.game_id,
                    opponent: op.opponent,
                    board_fen: board::INITIAL_BOARD.to_string(),
                    seq: 1,
                    player: match *player {
                        0 => Player::Black,
                        1 => Player::White,
                        _ => return Err(anyhow!("Invalid player enum")),
                    },
                    result: GameResult::None,
                };

                self.games.insert(game);
                self.save_games()?;
            },
            OperationType::MovePlayed { board, mv } => {
                if let Some(game_mut) = self.games.get_mut(op.game_id) {
                    // TODO: This requires a lot of honesty :P
                    game_mut.board_fen = board.clone();
                    self.save_games()?;
                    if let Screen::Game(ref mut screen) = self.active_screen {
                        if *screen.opponent() == op.opponent {
                            screen.apply_move(BitMove::new(*mv));
                        }
                    }
                }
            },
            OperationType::Resign => {
                if let Some(game_mut) = self.games.get_mut(op.game_id) {
                    game_mut.result = GameResult::TheyResigned;
                }
            },
        }

        Ok(())
    }

    fn save_games(&mut self) -> anyhow::Result<()> {
        self.games.clean_up();
        // TODO: decouple
        let json = serde_json::to_string(&self.games)?;
        fs::write(&self.config.save_path, json)?;
        Ok(())
    }
}

#[derive(Debug)]
enum Screen {
    Start(StartScreen),
    Game(GameScreen),
}

fn load_games<P: AsRef<Path>>(path: P) -> anyhow::Result<GameCollection> {
    let mut games = GameCollection::default();
    if path.as_ref().exists() {
        let mut read = fs::File::open(path)?;
        games = serde_json::from_reader(&mut read)?;
    }
    Ok(games)
}
