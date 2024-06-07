const COMMANDS: &[&str] = &["create_note"];

fn main() {
  fedimint_build::set_code_version();
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
