use crate::domain::hotkey::Action;
use crate::domain::repository;
use crate::domain::repository::LaunchError;

pub struct Launcher {}

impl repository::Launcher for Launcher {
    fn launch(&self, action: &Action) -> Result<(), LaunchError> {
        todo!()
    }
}
