use tauri::{plugin::{Builder, TauriPlugin}, Runtime};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("fedimint")
    .setup(|app, plugin| {
        // setup client data_dir
        // set out guardian peerId
        // password for authentication (should not be here?)
        // 
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
