use crate::app::App;
use crate::domain::hotkey::ShortcutId;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tracing::error;

pub struct EventListener {}

impl EventListener {
    pub fn handle_events(app: Arc<App>, rx: Receiver<ShortcutId>) {
        let core = app.clone();
        std::thread::spawn(move || {
            while let Ok(id) = rx.recv() {
                if let Err(e) = core.run_shortcut(&id) {
                    error!("run shortcut: {e}")
                }
            }
        });
    }
}
