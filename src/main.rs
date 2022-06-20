// #![windows_subsystem = "windows"]

use iced::{Application, Settings};
use sjqchatapp::app::App;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::run(Settings {
        ..Default::default()
    })?;
    Ok(())
}
