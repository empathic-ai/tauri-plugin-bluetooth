use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

use std::{collections::HashMap, sync::Mutex};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Bluetooth;
#[cfg(mobile)]
use mobile::Bluetooth;

#[derive(Default)]
struct MyState(Mutex<HashMap<String, String>>);

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the bluetooth APIs.
pub trait BluetoothExt<R: Runtime> {
  fn bluetooth(&self) -> &Bluetooth<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BluetoothExt<R> for T {
  fn bluetooth(&self) -> &Bluetooth<R> {
    self.state::<Bluetooth<R>>().inner()
  }
}

#[tauri::command]
async fn ping<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>, value: String) -> Result<String> {
  println!("foobar");
  Ok("HI FROM RUST!".into())
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  let mut builder = Builder::new("bluetooth");
  #[cfg(desktop)]
  {
    builder = builder.invoke_handler(tauri::generate_handler![ping]);
    println!("REGISTERED FOR DESKTOP!");
  }

  builder//.invoke_handler(tauri::generate_handler![commands::execute])
    .setup(|app, api| {
      #[cfg(mobile)]
      {
        let bluetooth = mobile::init(app, api)?;
        app.manage(bluetooth);
      }
      //#[cfg(desktop)]
      //let bluetooth = desktop::init(app, api)?;

      // manage state so it is accessible by the commands
      app.manage(MyState::default());
      Ok(())
    })
    .build()
}
