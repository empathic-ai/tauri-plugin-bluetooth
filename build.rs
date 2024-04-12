const COMMANDS: &[&str] = &[];

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  tauri_plugin::Builder::new(COMMANDS).build()
}