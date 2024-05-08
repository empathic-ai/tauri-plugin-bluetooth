use jni::{objects::GlobalRef, JNIEnv, AttachGuard, JavaVM};
use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

use std::{cell::RefCell, collections::HashMap, sync::{atomic::{AtomicUsize, Ordering}, Mutex, Weak}};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

use std::ffi::c_void;
use once_cell::sync::OnceCell;

#[cfg(desktop)]
use desktop::Bluetooth;
#[cfg(mobile)]
use mobile::Bluetooth;

use lazy_static::lazy_static;
use anyhow::anyhow;

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

pub static RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
static CLASS_LOADER: OnceCell<GlobalRef> = OnceCell::new();
pub static JAVAVM: OnceCell<JavaVM> = OnceCell::new();

std::thread_local! {
  static JNI_ENV: RefCell<Option<AttachGuard<'static>>> = RefCell::new(None);
}

pub fn create_runtime() -> anyhow::Result<()> {
  let vm = JAVAVM.get().ok_or(anyhow!("Failed to find Java VM!"))?;
  let env = vm.attach_current_thread().unwrap();

  // We create runtimes multiple times. Only run our loader setup once.
  if CLASS_LOADER.get().is_none() {
    setup_class_loader(&env).unwrap();
  }
  let runtime = {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .thread_name_fn(|| {
        static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
        let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
        format!("intiface-thread-{}", id)
      })
      .on_thread_stop(move || {
        JNI_ENV.with(|f| *f.borrow_mut() = None);
      })
      .on_thread_start(move || {
        // We now need to call the following code block via JNI calls. God help us.
        //
        //  java.lang.Thread.currentThread().setContextClassLoader(
        //    java.lang.ClassLoader.getSystemClassLoader()
        //  );
        let vm = JAVAVM.get().unwrap();
        let env = vm.attach_current_thread().unwrap();
        let thread = env
          .call_static_method(
            "java/lang/Thread",
            "currentThread",
            "()Ljava/lang/Thread;",
            &[],
          )
          .unwrap()
          .l()
          .unwrap();
        env
          .call_method(
            thread,
            "setContextClassLoader",
            "(Ljava/lang/ClassLoader;)V",
            &[CLASS_LOADER.get().unwrap().as_obj().into()],
          )
          .unwrap();
        JNI_ENV.with(|f| *f.borrow_mut() = Some(env));
      })
      .build()
      .unwrap()
  };

  RUNTIME.set(runtime).map_err(|_| anyhow!("Error mapping runtime (custom error)!"))?;
  Ok(())
}

fn setup_class_loader(env: &JNIEnv) -> anyhow::Result<()> {
  let thread = env
    .call_static_method(
      "java/lang/Thread",
      "currentThread",
      "()Ljava/lang/Thread;",
      &[],
    )?
    .l()?;
  let class_loader = env
    .call_method(
      thread,
      "getContextClassLoader",
      "()Ljava/lang/ClassLoader;",
      &[],
    )?
    .l()?;

  CLASS_LOADER
    .set(env.new_global_ref(class_loader)?)
    .map_err(|_| anyhow!("Class loader error (custom error)!"))
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: jni::JavaVM, _res: *const c_void) -> jni::sys::jint {

    let env = vm.get_env().unwrap();
    jni_utils::init(&env).unwrap();
    btleplug::platform::init(&env).unwrap();
    let _ = JAVAVM.set(vm);
    jni::JNIVersion::V6.into()
}

pub async fn ble_test() -> anyhow::Result<()> {

  create_runtime()
  .expect("Runtime should work, otherwise we can't function.");

  RUNTIME.get().unwrap().spawn(
    async move {
      ble_run().await.expect("Failed to run bluetooth test!");
    }
  );

  Ok(())
}

async fn ble_run() -> anyhow::Result<()> {
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
