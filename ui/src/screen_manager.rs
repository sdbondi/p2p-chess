use minifb::Window;
use p2p_chess_channel::{ChessOperation, MessageChannel, OperationType, TryRecvError};
use pleco::{BitMove, Player};
use tari_comms::types::CommsPublicKey;
use tari_utilities::hex::Hex;

use crate::{
    clipboard::Clipboard,
    color::Color,
    drawable::{Drawable, FrameBuffer},
    game::{GameConfig, GameScreen, GameStatus},
    start_screen::StartScreen,
};

#[derive(Debug)]
pub struct ScreenManager {
    config: GameConfig,
    active_screen: Screen,
    clipboard: Clipboard,
    channel: MessageChannel<ChessOperation>,
    // games: Vec<Game>,
}

impl ScreenManager {
    pub fn initialize(
        config: GameConfig,
        channel: MessageChannel<ChessOperation>,
        public_key: CommsPublicKey,
    ) -> anyhow::Result<Self> {
        let clipboard = Clipboard::initialize()?;

        Ok(Self {
            config,
            active_screen: Screen::Start(StartScreen::new(clipboard.clone(), public_key.to_hex())),
            clipboard,
            channel,
            // games: vec![],
        })
    }

    fn create_new_game(&mut self, pk: CommsPublicKey) {
        self.active_screen = Screen::Game(GameScreen::new(self.config.clone()));
        self.channel
            .try_send(ChessOperation {
                seq: 0,
                opponent: pk,
                operation: OperationType::NewGame {
                    player: Player::White as u8,
                },
            })
            .unwrap();
        // TODO
    }

    pub fn render(&mut self, window: &Window, buf: &mut FrameBuffer) {
        match self.active_screen {
            Screen::Start(ref mut main_screen) => {
                main_screen.draw(buf);
                main_screen.update(window);
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
            },
            Screen::Game(ref mut game) => {
                game.draw(buf);
                game.update(&window);
                match game.state().game_status() {
                    // TODO: display winner
                    GameStatus::StaleMate | GameStatus::CheckMate(_) | GameStatus::Resign(_) => {
                        buf.clear(Color::black());
                        self.active_screen = Screen::Start(StartScreen::new(self.clipboard.clone(), "TODO".into()));
                    },
                    _ => {},
                }
            },
        }

        match self.channel.try_recv() {
            Ok(evt) => {
                // screen_manager.apply_event(external_event);
            },
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {
                // TODO
            },
        }
    }

    pub fn apply_operation(&mut self, op: ChessOperation) {
        match self.active_screen {
            Screen::Start(ref mut screen) => match op.operation {
                OperationType::NewGame { .. } => {},
                OperationType::MovePlayed(mv) => {},
                OperationType::Resign => {},
            },
            Screen::Game(ref mut game) => match op.operation {
                OperationType::NewGame { .. } => {},
                OperationType::MovePlayed(mv) => {
                    game.apply_move(BitMove::new(mv));
                },
                OperationType::Resign => {},
            },
        }
    }
}

#[derive(Debug)]
enum Screen {
    Start(StartScreen),
    Game(GameScreen),
}
