use tauri::{AppHandle, command, Runtime, State, Window};

use crate::{MyState, error::TauriPluginResult};

#[command]
pub(crate) async fn create_notes<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState>,
) -> TauriPluginResult<String> {
  // state.0.lock().unwrap().insert("key".into(), "value".into());
  Ok("success".to_string())
}
