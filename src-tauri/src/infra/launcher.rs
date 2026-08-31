use crate::domain::hotkey::Action;
use crate::domain::repository;
use crate::domain::repository::LaunchError;
use std::process::Command;
use std::sync::Arc;

pub struct Launcher {}

impl Launcher {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }
}

impl repository::Launcher for Launcher {
    fn launch(&self, action: &Action) -> Result<(), LaunchError> {
        match action {
            Action::Launch { program, args } => {
                let mut cmd = Command::new(program);
                cmd.args(args);

                #[cfg(target_os = "windows")]
                {
                    // CREATE_NO_WINDOW
                    const CREATE_NO_WINDOW: u32 = 0x08000000;
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(CREATE_NO_WINDOW);
                }

                cmd.spawn()
                    .map(|_| ())
                    .map_err(|e| LaunchError::Internal(e.to_string()))
            }
        }
    }
}
