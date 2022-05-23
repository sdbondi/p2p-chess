use std::{
    fs,
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::anyhow;
use minifb::Window;
use p2p_chess_channel::{ChessOperation, MessageChannel, OperationType, TryRecvError, TrySendError};
use pleco::{BitMove, Player};
use rand::{rngs::OsRng, RngCore};
use tari_comms::types::CommsPublicKey;
use tari_crypto::tari_utilities::encoding::Base58;

use crate::{
    board,
    clipboard::Clipboard,
    color::Color,
    drawable::{Drawable, FrameBuffer},
    game::{Game, GameCollection, GameResult},
    game_screen::{GameConfig, GameScreen},
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
    last_sync: Instant,
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
            last_sync: Instant::now(),
        })
    }

    fn create_new_game(&mut self, opponent: CommsPublicKey) {
        // TODO: allow player to choose black/white
        let id = OsRng.next_u32();
        self.active_screen = Screen::Game(GameScreen::new(
            id,
            0,
            self.config.clone(),
            Player::White,
            opponent.clone(),
            board::INITIAL_BOARD,
        ));
        self.games.insert(Game {
            id,
            opponent: opponent.clone(),
            board_fen: board::INITIAL_BOARD.to_string(),
            seq: 0,
            player: Player::White,
            result: GameResult::None,
            last_activity: current_timestamp(),
        });
        self.save_games().unwrap();
        self.send_message(ChessOperation {
            game_id: id,
            seq: 0,
            to: opponent,
            from: self.public_key.clone(),
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
                self.games.sort();
                main_screen.set_games(&self.games);
                main_screen.draw(buf);
                let idx = main_screen.show_game_clicked();
                if let Some(pk) = main_screen.new_game_clicked() {
                    match CommsPublicKey::from_base58(pk) {
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
                    let game = &self.games[idx];
                    dbg!(game);
                    buf.clear(Color::black());
                    // TODO: clean up game state management in general - just rushing to be able to play this right now
                    self.active_screen = Screen::Game(GameScreen::new(
                        game.id,
                        game.seq,
                        self.config.clone(),
                        game.player,
                        game.opponent.clone(),
                        &game.board_fen,
                    ));
                }
            },
            Screen::Game(ref mut game) => {
                game.draw(buf);
                game.update(&window);
                if let Some(mv) = game.take_last_move_played() {
                    dbg!("move played", mv);
                    let msg = ChessOperation {
                        game_id: game.game_id(),
                        seq: game.next_seq(),
                        to: game.opponent().clone(),
                        from: self.public_key.clone(),
                        operation: OperationType::MovePlayed {
                            mv: mv.get_raw(),
                            board: game.to_board_fen(),
                        },
                    };
                    self.channel.try_send(msg).unwrap();
                    if let Some(game_mut) = self.games.get_mut(game.game_id()) {
                        game_mut.board_fen = game.to_board_fen();
                        game_mut.last_activity = current_timestamp();
                    }
                }

                if game.was_back_clicked() {
                    buf.clear(Color::black());
                    self.active_screen =
                        Screen::Start(StartScreen::new(self.clipboard.clone(), self.public_key.clone()));
                } else if self.last_sync.elapsed() > Duration::from_secs(30) {
                    let msg = ChessOperation {
                        game_id: game.game_id(),
                        seq: game.seq(),
                        to: game.opponent().clone(),
                        from: self.public_key.clone(),
                        operation: OperationType::Sync {
                            board: game.to_board_fen(),
                        },
                    };
                    self.send_message(msg).unwrap();
                    self.last_sync = Instant::now();
                }
            },
        }

        if let Err(err) = self.save_games() {
            log::error!("save failed: {}", err);
        }

        match self.channel.try_recv() {
            Ok(op) => {
                self.last_sync = Instant::now();
                dbg!(&op);
                if let Err(err) = self.apply_operation(op) {
                    log::error!("apply operation failed: {}", err);
                }
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {
                // TODO
                panic!("channel disconnected");
            },
        }
    }

    pub fn apply_operation(&mut self, op: ChessOperation) -> anyhow::Result<()> {
        match &op.operation {
            OperationType::NewGame { player } => {
                let game = Game {
                    id: op.game_id,
                    opponent: op.from,
                    board_fen: board::INITIAL_BOARD.to_string(),
                    seq: 0,
                    player: match *player {
                        0 => Player::White,
                        1 => Player::Black,
                        _ => return Err(anyhow!("Invalid player enum")),
                    },
                    result: GameResult::None,
                    last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                };

                self.games.insert(game);
                self.save_games()?;
            },
            OperationType::MovePlayed { board, mv } => {
                dbg!(board, mv);
                if let Some(game_mut) = self.games.get_mut(op.game_id) {
                    if op.seq <= game_mut.seq {
                        dbg!("ignore move", op.seq, game_mut.seq);
                        return Ok(());
                    }
                    // TODO: This requires a lot of honesty :P
                    game_mut.board_fen = board.clone();
                    game_mut.seq = op.seq;
                    game_mut.last_activity = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    self.save_games()?;
                    if let Screen::Game(ref mut screen) = self.active_screen {
                        dbg!(screen.game_id(), op.game_id);
                        if screen.game_id() == op.game_id {
                            let mv = BitMove::new(*mv);
                            log::info!("Move played for active game {}", mv);
                            screen.set_board_state(&board, Some(mv)).set_seq(op.seq);
                        }
                    }
                }
            },
            OperationType::Resign => {
                if let Some(game_mut) = self.games.get_mut(op.game_id) {
                    game_mut.result = GameResult::TheyResigned;
                }
            },
            OperationType::Sync { board } => {
                if let Some(game_mut) = self.games.get_mut(op.game_id) {
                    match op.seq {
                        seq if seq < game_mut.seq => {
                            // Send a message back with our state
                            let msg = ChessOperation {
                                game_id: game_mut.id,
                                seq: game_mut.seq,
                                to: game_mut.opponent.clone(),
                                from: self.public_key.clone(),
                                operation: OperationType::Sync {
                                    board: game_mut.board_fen.clone(),
                                },
                            };
                            self.send_message(msg).unwrap();
                        },
                        seq if seq == game_mut.seq => {},
                        _ => {
                            // Update our game state
                            game_mut.board_fen = board.clone();
                            game_mut.seq = op.seq;
                            game_mut.last_activity = current_timestamp();
                            self.active_screen.refresh_game(&*game_mut);
                            self.save_games()?;
                        },
                    }
                }
            },
        }

        Ok(())
    }

    fn send_message(&mut self, msg: ChessOperation) -> Result<(), TrySendError<ChessOperation>> {
        self.channel.try_send(msg)
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

impl Screen {
    pub fn refresh_game(&mut self, game: &Game) {
        match self {
            Screen::Start(_) => {},
            Screen::Game(g) => {
                if g.game_id() == game.id {
                    g.set_seq(game.seq);
                    g.set_board_state(&game.board_fen, None);
                }
            },
        }
    }
}

fn load_games<P: AsRef<Path>>(path: P) -> anyhow::Result<GameCollection> {
    let mut games = GameCollection::default();
    if path.as_ref().exists() {
        let mut read = fs::File::open(path)?;
        games = serde_json::from_reader(&mut read)?;
    }
    Ok(games)
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}
