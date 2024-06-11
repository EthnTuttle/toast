use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::error::TauriPluginResult<Roastr<R>> {
  Ok(Roastr(app.clone()))
}

/// Access to the roastr APIs.
pub struct Roastr<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Roastr<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::error::TauriPluginResult<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
