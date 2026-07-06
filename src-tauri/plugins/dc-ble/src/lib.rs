// AI-generated (Claude)
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod error;
mod mobile;
mod models;

pub use error::{Error, Result};
pub use mobile::DcBle;
pub use models::*;

pub trait DcBleExt<R: Runtime> {
    fn dc_ble(&self) -> &DcBle<R>;
}

impl<R: Runtime, T: Manager<R>> DcBleExt<R> for T {
    fn dc_ble(&self) -> &DcBle<R> {
        self.state::<DcBle<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("dc-ble")
        .setup(|app, api| {
            let dc_ble = mobile::init(app, api)?;
            app.manage(dc_ble);
            Ok(())
        })
        .build()
}
