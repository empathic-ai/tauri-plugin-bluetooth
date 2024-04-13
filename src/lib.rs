// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(not(target_arch = "wasm32"))]
#[cfg(desktop)]
mod desktop;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(mobile)]
mod mobile;

#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod models;

#[cfg(not(target_arch = "wasm32"))]
mod main {
  use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
  };

  pub use crate::models::*;

  #[cfg(desktop)]
  use crate::desktop::Sample;
  #[cfg(mobile)]
  use crate::mobile::Sample;

  pub use crate::error::*;

  /// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sample APIs.
  pub trait SampleExt<R: Runtime> {
    fn sample(&self) -> &Sample<R>;
  }

  impl<R: Runtime, T: Manager<R>> SampleExt<R> for T {
    fn sample(&self) -> &Sample<R> {
      self.state::<Sample<R>>().inner()
    }
  }

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("bluetooth")
      .setup(|app, api| {
        #[cfg(mobile)]
        let sample = crate::mobile::init(app, api)?;
        #[cfg(desktop)]
        let sample = crate::desktop::init(app, api)?;
        app.manage(sample);

        Ok(())
      })
      .on_navigation(|window, url| {
        println!("navigation {} {url}", window.label());
        true
      })
      .build()
  }
}

#[cfg(target_arch = "wasm32")]
mod main {
  
}

pub use main::*;