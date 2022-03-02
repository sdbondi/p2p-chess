use crate::color::Color;
use crate::command::{ChessCommand, CommandPublisher, CommandSubscription};
use crate::drawable::FrameBuffer;
use crate::game::GameConfig;
use crate::screen_manager::ScreenManager;
use commands::{ChessOperation, CommandPublisher, CommandSubscription};
use minifb::{Key, Window, WindowOptions};
use std::sync::mpsc;
use std::time::Duration;
use tari_common_types::types::PublicKey;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

const BACKGROUND_COLOUR: Color = Color::black();

pub struct ChessUi {
    title: &'static str,
    window_width: usize,
    window_height: usize,
    opts: WindowOptions,
    publisher: CommandPublisher,
    inbound_commands: CommandReceiver,
    public_key: PublicKey,
}

impl ChessUi {
    pub fn new(
        title: &'static str,
        window_width: usize,
        window_height: usize,
        opts: WindowOptions,
        inbound_commands: CommandReceiver,
        public_key: PublicKey,
    ) -> Self {
        Self {
            title,
            window_width,
            window_height,
            opts,
            publisher: CommandPublisher::new(),
            inbound_commands: inbound_commands,
            public_key,
        }
    }

    pub fn subscribe(&self) -> CommandSubscription {
        self.publisher.subscribe()
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let mut window = Window::new(self.title, self.window_width, self.window_height, self.opts)?;

        // ~60fps
        window.limit_update_rate(Some(Duration::from_micros(16600)));

        self.ui_loop(window)?;

        Ok(())
    }

    fn ui_loop(&mut self, mut window: Window) -> anyhow::Result<()> {
        let mut buf = FrameBuffer::new(
            self.window_width as u32,
            self.window_height as u32,
            BACKGROUND_COLOUR,
        );

        let config = GameConfig {
            window_width: self.window_width as u32,
            window_height: self.window_height as u32,
            light_color: Color::cream(),
            dark_color: Color::dark_green(),
        };

        let mut screen_manager = ScreenManager::initialize(self.publisher.clone(), config)?;
        let mut should_exit = false;
        while window.is_open() && !should_exit {
            screen_manager.render(&window, &mut buf);
            match self.inbound_commands.try_recv() {
                Ok(evt) => {
                    screen_manager.apply_event(external_event);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    should_exit = true;
                }
            };

            if let Some(keys) = window.get_keys() {
                if (keys.contains(&Key::LeftCtrl) || keys.contains(&Key::RightCtrl))
                    && keys.contains(&Key::Q)
                {
                    should_exit = true;
                }
            }

            window.update_with_buffer(buf.as_slice(), self.window_width, self.window_height)?;
        }
        Ok(())
    }
}
