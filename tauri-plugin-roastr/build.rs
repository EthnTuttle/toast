const COMMANDS: &[&str] = &["join_federation_as_admin"];

fn main() {
  fedimint_build::set_code_version();
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
