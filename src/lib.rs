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

use std::time::Duration;
use tokio::time;

use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter};
type BtleManager = btleplug::platform::Manager;

#[tauri::command]
async fn ping<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>, value: String) -> Result<String> {
  println!("foobar");

  ble_test().await.expect("Failed to run ble test!");
  Ok("HI FROM RUST!".into())
}

pub async fn ble_test() -> anyhow::Result<()> {

  let manager = BtleManager::new().await?;
  let adapter_list = manager.adapters().await?;
  if adapter_list.is_empty() {
      eprintln!("No Bluetooth adapters found");
  }

  for adapter in adapter_list.iter() {
      println!("Starting scan on {}...", adapter.adapter_info().await?);
      adapter
          .start_scan(ScanFilter::default())
          .await
          .expect("Can't scan BLE adapter for connected devices...");
      time::sleep(Duration::from_secs(10)).await;
      let peripherals = adapter.peripherals().await?;
      if peripherals.is_empty() {
          eprintln!("->>> BLE peripheral devices were not found, sorry. Exiting...");
      } else {
          // All peripheral devices in range
          for peripheral in peripherals.iter() {
              let properties = peripheral.properties().await?;
              let is_connected = peripheral.is_connected().await?;
              let local_name = properties
                  .unwrap()
                  .local_name
                  .unwrap_or(String::from("(peripheral name unknown)"));
              println!(
                  "Peripheral {:?} is connected: {:?}",
                  local_name, is_connected
              );
              if !is_connected {
                  println!("Connecting to peripheral {:?}...", &local_name);
                  if let Err(err) = peripheral.connect().await {
                      eprintln!("Error connecting to peripheral, skipping: {}", err);
                      continue;
                  }
              }
              let is_connected = peripheral.is_connected().await?;
              println!(
                  "Now connected ({:?}) to peripheral {:?}...",
                  is_connected, &local_name
              );
              peripheral.discover_services().await?;
              println!("Discover peripheral {:?} services...", &local_name);
              for service in peripheral.services() {
                  println!(
                      "Service UUID {}, primary: {}",
                      service.uuid, service.primary
                  );
                  for characteristic in service.characteristics {
                      println!("  {:?}", characteristic);
                  }
              }
              if is_connected {
                  println!("Disconnecting from peripheral {:?}...", &local_name);
                  peripheral
                      .disconnect()
                      .await
                      .expect("Error disconnecting from BLE peripheral");
              }
          }
      }
  }
  Ok(())
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  let mut builder = Builder::new("bluetooth");
  //#[cfg(desktop)]
  //{
    builder = builder.invoke_handler(tauri::generate_handler![ping]);
    println!("REGISTERED BLUETOOTH PLUGIN COMMAND!");
  //}

  builder//.invoke_handler(tauri::generate_handler![commands::execute])
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
