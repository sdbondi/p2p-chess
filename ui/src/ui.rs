use std::{path::PathBuf, time::Duration};

use minifb::{Key, Window, WindowOptions};
use p2p_chess_channel::{ChessOperation, MessageChannel};
use tari_comms::types::CommsPublicKey;

use crate::{color::Color, drawable::FrameBuffer, game_screen::GameConfig, screen_manager::ScreenManager};

const BACKGROUND_COLOUR: Color = Color::black();

pub struct ChessUi {
    title: &'static str,
    window_width: usize,
    window_height: usize,
    opts: WindowOptions,
    channel: MessageChannel<ChessOperation>,
    public_key: CommsPublicKey,
    base_path: PathBuf,
}

impl ChessUi {
    pub fn new(
        title: &'static str,
        window_width: usize,
        window_height: usize,
        opts: WindowOptions,
        // TODO: Cleanup - decouple game loading/saving backend from UI
        base_path: PathBuf,
        channel: MessageChannel<ChessOperation>,
        public_key: CommsPublicKey,
    ) -> Self {
        Self {
            title,
            window_width,
            window_height,
            opts,
            channel,
            base_path,
            public_key,
        }
    }

    pub fn run(self) -> anyhow::Result<()> {
        let mut window = Window::new(self.title, self.window_width, self.window_height, self.opts)?;

        // ~60fps
        window.limit_update_rate(Some(Duration::from_micros(16600)));

        self.ui_loop(window)?;

        Ok(())
    }

    fn ui_loop(self, mut window: Window) -> anyhow::Result<()> {
        let mut buf = FrameBuffer::new(self.window_width as u32, self.window_height as u32, BACKGROUND_COLOUR);

        let config = GameConfig {
            window_width: self.window_width as u32,
            window_height: self.window_height as u32,
            light_color: Color::cream(),
            dark_color: Color::dark_green(),
            save_path: self.base_path.join("p2pc-games.json"),
        };

        let mut screen_manager = ScreenManager::initialize(config, self.channel, self.public_key)?;

        while window.is_open() {
            screen_manager.render(&window, &mut buf);

            if let Some(keys) = window.get_keys() {
                if (keys.contains(&Key::LeftCtrl) || keys.contains(&Key::RightCtrl)) && keys.contains(&Key::Q) {
                    break;
                }
            }

            window.update_with_buffer(buf.as_slice(), self.window_width, self.window_height)?;
        }
        Ok(())
    }
}
