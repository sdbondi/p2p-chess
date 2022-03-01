use tokio::runtime::Runtime;
use ui::{ChessUi, ScaleMode, WindowOptions};

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;

fn main() -> anyhow::Result<()> {
    let runtime = Runtime::new()?;
    let ui = ChessUi::new(
        "P2P Chess",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        WindowOptions {
            title: true,
            scale_mode: ScaleMode::Center,
            resize: true,
            ..Default::default()
        },
    );

    let mut ui_subscription = ui.subscribe();
    let state = ui.shared_state();

    runtime.spawn(async move {
        while let Ok(event) = ui_subscription.recv().await {
            println!("{:?}", event);
            state.start_new_game(Player::White);
        }
    });

    ui.run()?;

    Ok(())
}
