use crate::app::App;
use std::sync::{mpsc, Arc};
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Shortcut, ShortcutEvent, ShortcutState};
use tracing::error;

pub fn hotkey_handler(
    tx: mpsc::Sender<u32>,
) -> impl Fn(&AppHandle, &Shortcut, ShortcutEvent) + Send + Sync + 'static {
    move |_app, shortcut, event| {
        if event.state() != ShortcutState::Pressed {
            return;
        }
        let _ = tx.send(shortcut.id);
    }
}

pub fn spawn_worker(rx: mpsc::Receiver<u32>, core: Arc<App>) {
    std::thread::Builder::new()
        .name("hotkey-worker".to_string())
        .spawn(move || {
            while let Ok(os_id) = rx.recv() {
                if let Err(e) = core.run_shortcut(&os_id) {
                    error!("run shortcut: {e}");
                };
            }
        })
        .expect("spawn hotkey worker");
}
