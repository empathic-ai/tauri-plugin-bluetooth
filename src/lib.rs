/*
#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
#[cfg(desktop)]
mod desktop;
//#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
//#[cfg(mobile)]
//mod mobile;
//#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
//mod commands;
#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
mod error;
//#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
//mod models;
//#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
//mod handler;

#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
mod lib {
  
  use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
  };
  
  //pub use crate::models::*;
  
  pub use crate::error::{Error, Result};

  #[cfg(desktop)]
  use crate::desktop::Bluetooth;
  //#[cfg(mobile)]
  //use crate::mobile::Bluetooth;

  //pub use crate::handler::Handler;

  #[derive(Default)]
  pub struct MyState(pub Mutex<HashMap<String, String>>);
  
  /*
  /// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the bluetooth APIs.
  pub trait BluetoothExt<R: Runtime> {
    fn bluetooth(&self) -> &Bluetooth<R>;
  }
  
  impl<R: Runtime, T: Manager<R>> crate::BluetoothExt<R> for T {
    fn bluetooth(&self) -> &Bluetooth<R> {
      self.state::<Bluetooth<R>>().inner()
    }
  } */
  
  use std::{collections::HashMap, sync::Mutex, time::Duration};
  use tokio::time;

  //use crate::{commands, handler, models};
  
  /// Initializes the plugin.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let mut builder = Builder::new("bluetooth");
    //#[cfg(desktop)]
    //{
      //builder = builder.invoke_handler(tauri::generate_handler![ping]);
      //println!("REGISTERED BLUETOOTH PLUGIN COMMAND!");
    //}


    builder//.invoke_handler(tauri::generate_handler![commands::execute])
      //.invoke_handler(commands::commands())
      .setup(|app, api| {
        //#[cfg(mobile)]
        //{
        //  let bluetooth = mobile::init(app, api)?;
        //  app.manage(bluetooth);
        //}
        //#[cfg(desktop)]
        //let bluetooth = desktop::init(app, api)?;
  
        // manage state so it is accessible by the commands
        app.manage(MyState::default());
        Ok(())
      })
      .build()
  }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_arch = "xtensa")))]
pub use lib::*;

/*
mod handler;

pub use error::Error;
pub use handler::Handler;

static HANDLER: OnceCell<Handler> = OnceCell::new();

/// Initializes the plugin.
/// # Panics
/// Panics if the handler cannot be initialized.
pub fn init() -> TauriPlugin<Wry> {
    let handler = async_runtime::block_on(Handler::new()).expect("failed to initialize handler");
    let _ = HANDLER.set(handler);

    #[allow(unused)]
    Builder::new("blec")
        .invoke_handler(commands::commands())
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            android::init(app, api)?;
            async_runtime::spawn(handle_events());
            Ok(())
        })
        .build()
}

/// Returns the BLE handler to use blec from rust.
/// # Errors
/// Returns an error if the handler is not initialized.
pub fn get_handler() -> error::Result<&'static Handler> {
    let handler = HANDLER.get().ok_or(error::Error::HandlerNotInitialized)?;
    Ok(handler)
}

async fn handle_events() {
    let handler = get_handler().expect("failed to get handler");
    let stream = handler
        .get_event_stream()
        .await
        .expect("failed to get event stream");
    stream
        .for_each(|event| async {
            handler
                .handle_event(event)
                .await
                .expect("failed to handle event");
        })
        .await;
}
 */
 */