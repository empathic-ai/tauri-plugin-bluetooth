const COMMANDS: &[&str] = &["ping"];

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  tauri_plugin::Builder::new(COMMANDS)
    //.global_api_script_path("./api-iife.js")
    .android_path("android")
    .ios_path("ios")
    .build();
}