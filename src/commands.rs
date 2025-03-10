/*
use tauri::{AppHandle, command, Runtime, State, Window};

use crate::{MyState, Result};

#[command]
pub(crate) async fn execute<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState>,
) -> Result<String> {
  state.0.lock().unwrap().insert("key".into(), "value".into());
  Ok("success".to_string())
}
*/
use std::{cell::RefCell, collections::HashMap, sync::{atomic::{AtomicUsize, Ordering}, Mutex, Weak}, time::Duration};
  
use std::ffi::c_void;
use btleplug::{api::{Central, Manager as _, Peripheral, ScanFilter}, platform::{Adapter, Manager}};

#[cfg(target_os = "android")]
use jni::{objects::GlobalRef, JNIEnv, AttachGuard, JavaVM};

use once_cell::sync::OnceCell;
use lazy_static::lazy_static;
use anyhow::anyhow;

use tauri::ipc::Channel;
use tauri::{async_runtime, command, AppHandle, Runtime};
use tokio::{sync::mpsc, time};
use tracing::info;
use uuid::Uuid;

use crate::error::Result;
use crate::get_handler;
use crate::models::{BleDevice, ScanFilter, WriteType};

static MANAGER: OnceCell<Manager> = OnceCell::new();

pub fn get_manager<'a>() -> &'a Manager {
  let handler = MANAGER.get().unwrap();
  handler
}

pub fn create_manager() {

  let manager = get_runtime().block_on(async move {
    BtleManager::new().await.expect("Failed to get BLE manager")
  });

  let _ = MANAGER.set(manager);
}

#[command]
pub(crate) async fn scan<R: Runtime>(
    _app: AppHandle<R>,
    timeout: u64,
    on_devices: Channel<Vec<BleDevice>>,
) -> Result<()> {
    tracing::info!("Scanning for BLE devices");
    let handler = get_handler()?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    async_runtime::spawn(async move {
        while let Some(devices) = rx.recv().await {
            on_devices
                .send(devices)
                .expect("failed to send device to the front-end");
        }
    });
    handler
        .discover(Some(tx), timeout, ScanFilter::None)
        .await?;
    Ok(())
}
/*
#[command]
pub(crate) async fn scan<R: Runtime>(
    _app: AppHandle<R>,
    timeout: u64,
    on_devices: Channel<Vec<BleDevice>>,
) -> Result<()> {

  get_runtime().spawn(async move {
    let manager = get_manager();
    let adapter = manager.adapters().await?.first().unwrap();
    //if adapter_list.is_empty() {
    //    eprintln!("No Bluetooth adapters found");
    //}
  
    //for adapter in adapter_list.iter() {
    println!("Starting scan on {}...", adapter.adapter_info().await?);
    adapter
        .start_scan(ScanFilter::default())
        .await
        .expect("Can't scan BLE adapter for connected devices...");
    time::sleep(Duration::from_secs(10)).await;
    Ok(())
  }).await.expect("Failed to scan for BLE devices.");

  Ok(())
}
*/

#[command]
pub(crate) async fn stop_scan<R: Runtime>(_app: AppHandle<R>) -> Result<()> {
    tracing::info!("Stopping BLE scan");
    let handler = get_handler()?;
    handler.stop_scan().await?;
    Ok(())
}

#[command]
pub(crate) async fn connect<R: Runtime>(
    _app: AppHandle<R>,
    address: String,
    on_disconnect: Channel<()>,
) -> Result<()> {
    tracing::info!("Connecting to BLE device: {:?}", address);
    let handler = get_handler()?;
    let disconnct_handler = move || {
        on_disconnect
            .send(())
            .expect("failed to send disconnect event to the front-end");
    };
    handler
        .connect(&address, Some(Box::new(disconnct_handler)))
        .await?;
    Ok(())
}

#[command]
pub(crate) async fn disconnect<R: Runtime>(_app: AppHandle<R>) -> Result<()> {
    tracing::info!("Disconnecting from BLE device");
    let handler = get_handler()?;
    handler.disconnect().await?;
    Ok(())
}

#[command]
pub(crate) async fn connection_state<R: Runtime>(
    _app: AppHandle<R>,
    update: Channel<bool>,
) -> Result<()> {
    let handler = get_handler()?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    handler.set_connection_update_channel(tx).await;
    update
        .send(handler.is_connected())
        .expect("failed to send connection state");
    async_runtime::spawn(async move {
        while let Some(connected) = rx.recv().await {
            update
                .send(connected)
                .expect("failed to send connection state to the front-end");
        }
    });
    Ok(())
}

#[command]
pub(crate) async fn scanning_state<R: Runtime>(
    _app: AppHandle<R>,
    update: Channel<bool>,
) -> Result<()> {
    let handler = get_handler()?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    handler.set_scanning_update_channel(tx).await;
    update
        .send(handler.is_scanning().await)
        .expect("failed to send scanning state");
    async_runtime::spawn(async move {
        while let Some(scanning) = rx.recv().await {
            update
                .send(scanning)
                .expect("failed to send scanning state to the front-end");
        }
    });
    Ok(())
}

#[command]
pub(crate) async fn send<R: Runtime>(
    _app: AppHandle<R>,
    characteristic: Uuid,
    data: Vec<u8>,
    write_type: WriteType,
) -> Result<()> {
    info!("Sending data: {data:?}");
    let handler = get_handler()?;
    handler.send_data(characteristic, &data, write_type).await?;
    Ok(())
}

#[command]
pub(crate) async fn recv<R: Runtime>(_app: AppHandle<R>, characteristic: Uuid) -> Result<Vec<u8>> {
    let handler = get_handler()?;
    let data = handler.recv_data(characteristic).await?;
    Ok(data)
}

#[command]
pub(crate) async fn send_string<R: Runtime>(
    app: AppHandle<R>,
    characteristic: Uuid,
    data: String,
    write_type: WriteType,
) -> Result<()> {
    let data = data.as_bytes().to_vec();
    send(app, characteristic, data, write_type).await
}

#[command]
pub(crate) async fn recv_string<R: Runtime>(
    app: AppHandle<R>,
    characteristic: Uuid,
) -> Result<String> {
    let data = recv(app, characteristic).await?;
    Ok(String::from_utf8(data).expect("failed to convert data to string"))
}

async fn subscribe_channel(characteristic: Uuid) -> Result<mpsc::Receiver<Vec<u8>>> {
    let handler = get_handler()?;
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    handler
        .subscribe(characteristic, move |data| {
            info!("subscribe_channel: {:?}", data);
            tx.try_send(data.to_vec())
                .expect("failed to send data to the channel");
        })
        .await?;
    Ok(rx)
}
#[command]
pub(crate) async fn subscribe<R: Runtime>(
    _app: AppHandle<R>,
    characteristic: Uuid,
    on_data: Channel<Vec<u8>>,
) -> Result<()> {
    let mut rx = subscribe_channel(characteristic).await?;
    async_runtime::spawn(async move {
        while let Some(data) = rx.recv().await {
            on_data
                .send(data)
                .expect("failed to send data to the front-end");
        }
    });
    Ok(())
}

#[command]
pub(crate) async fn subscribe_string<R: Runtime>(
    _app: AppHandle<R>,
    characteristic: Uuid,
    on_data: Channel<String>,
) -> Result<()> {
    let mut rx = subscribe_channel(characteristic).await?;
    async_runtime::spawn(async move {
        while let Some(data) = rx.recv().await {
            info!("subscribe_string: {:?}", data);
            let data = String::from_utf8(data).expect("failed to convert data to string");
            on_data
                .send(data)
                .expect("failed to send data to the front-end");
        }
    });
    Ok(())
}

#[command]
pub(crate) async fn unsubscribe<R: Runtime>(
    _app: AppHandle<R>,
    characteristic: Uuid,
) -> Result<()> {
    let handler = get_handler()?;
    handler.unsubscribe(characteristic).await?;
    Ok(())
}

pub fn commands<R: Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool {
    tauri::generate_handler![
        scan,
        stop_scan,
        connect,
        disconnect,
        connection_state,
        send,
        send_string,
        recv,
        recv_string,
        subscribe,
        subscribe_string,
        unsubscribe,
        scanning_state
    ]
}

type BtleManager = btleplug::platform::Manager;
  
#[tauri::command]
async fn ping<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>, value: String) -> Result<String> {
  println!("foobar");

  ble_test().await.expect("Failed to run ble test!");
  Ok("HI FROM RUST!".into())
}

pub static RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {

  create_runtime()
  .expect("Runtime should work, otherwise we can't function.");

  RUNTIME.get().unwrap()
  /* .spawn(
    async move {
      ble_run().await//.expect("Failed to run bluetooth test!");
    }
  ).await??;
  */
}

#[cfg(target_os = "android")]
static CLASS_LOADER: OnceCell<jni::objects::GlobalRef> = OnceCell::new();

#[cfg(target_os = "android")]
pub static JAVAVM: OnceCell<jni::sys::JavaVM> = OnceCell::new();

#[cfg(target_os = "android")]
std::thread_local! {
  static JNI_ENV: RefCell<Option<AttachGuard<'static>>> = RefCell::new(None);
}

#[cfg(not(target_os = "android"))]
pub fn create_runtime() -> anyhow::Result<()> {
  let runtime = {
    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .thread_name_fn(|| {
        static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
        let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
        format!("intiface-thread-{id}")
      })
      .on_thread_start(|| {})
      .build()
      .unwrap()
  };

  RUNTIME.set(runtime).map_err(|_| anyhow!("Error mapping runtime (custom error)!"))?;
  Ok(())
}

#[cfg(target_os = "android")]
pub fn create_runtime() -> anyhow::Result<()> {
  let vm = JAVAVM.get().ok_or(anyhow::anyhow!("Failed to find Java VM!"))?;
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
        println!("JNI Thread stopped");
        JNI_ENV.with(|f| *f.borrow_mut() = None);
      })
      .on_thread_start(move || {
        println!("JNI Thread started");
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

#[cfg(target_os = "android")]
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