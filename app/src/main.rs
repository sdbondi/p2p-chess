use tokio::runtime::Runtime;
use ui::events::EventSubscription;
use ui::{ChessUi, ScaleMode, WindowOptions};

const WINDOW_WIDTH: usize = 1024;
const WINDOW_HEIGHT: usize = 90 * 8;

fn main() -> anyhow::Result<()> {
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

    spawn_network_worker(ui_subscription, state);

    ui.run()?;

    Ok(())
}

fn spawn_network_worker(ui_events: EventSubscription, state: SharedState) {
    let runtime = Runtime::new()?;

    runtime.spawn(async move {
        while let Ok(event) = ui_subscription.recv().await {
            println!("{:?}", event);
            state.start_new_game(Player::White);
        }
    });
}
