use tauri::{async_runtime::Mutex, plugin::{Builder, TauriPlugin}, Manager, Runtime};
use fedimint_client::module::init::ClientModuleInitRegistry;

struct Config {
  module_inits: Mutex<ClientModuleInitRegistry>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("fedimint")
    .setup(|app, plugin| {
        // setup client data_dir
        // set our guardian peerId
        // password for authentication (should not be here?)
        // check version hashes
        app.manage(Config { module_inits: Default::default() });
        Ok(())
    })
    .build()
}


#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
