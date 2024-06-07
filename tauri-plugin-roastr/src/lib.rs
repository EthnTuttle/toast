use fedimint_core::{apply, async_trait_maybe_send, db::Database, PeerId};
use fedimint_client::{module::init::ClientModuleInitRegistry, Client, ClientHandle};
use roastr_client::RoastrClientInit;
use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

use std::path::PathBuf;

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Roastr;
#[cfg(mobile)]
use mobile::Roastr;

struct MyState {
  client: ClientHandle,
  our_peer_id: Option<PeerId>,
  password: Option<String>
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the roastr APIs.
pub trait RoastrExt<R: Runtime> {
  fn roastr(&self) -> &Roastr<R>;
}

impl<R: Runtime, T: Manager<R>> crate::RoastrExt<R> for T {
  fn roastr(&self) -> &Roastr<R> {
    self.state::<Roastr<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("roastr")
    .invoke_handler(tauri::generate_handler![commands::execute])
    .setup(|app, api| {
      #[cfg(mobile)]
      let roastr = mobile::init(app, api)?;
      #[cfg(desktop)]
      let roastr = desktop::init(app, api)?;
      app.manage(roastr);
      let mut module_inits = ClientModuleInitRegistry::new();
      module_inits.attach(RoastrClientInit);

      // setup data-dir
      // setup db

      let client_builder = Client::builder(db);
      // manage state so it is accessible by the commands
      app.manage(MyState { client: todo!(), our_peer_id: todo!(), password: todo!()  });
      Ok(())
    })
    .build()
}

async fn load_rocks_db(&self) -> Result<Database> {
    debug!(target: LOG_CLIENT, "Loading client database");
    let db_path = self.data_dir_create().await?.join("client.db");
    let lock_path = db_path.with_extension("db.lock");
    Ok(LockedBuilder::new(&lock_path)
        .await
        .map_err_cli_msg("could not lock database")?
        .with_db(
            fedimint_rocksdb::RocksDb::open(db_path)
                .map_err_cli_msg("could not open database")?,
        )
        .into())
}


// #### db_locked.rs from Fedimint CLI
use std::path::Path;

use anyhow::Context;
use fedimint_core::db::IRawDatabase;
use fedimint_logging::LOG_CLIENT;
use tracing::{debug, info};

/// Locked version of database
///
/// This will use file-system advisory locks to prevent to
/// serialize opening and using the `DB`.
///
/// Use [`LockedBuilder`] to create.
#[derive(Debug)]
pub struct Locked<DB> {
    inner: DB,
    #[allow(dead_code)] // only for `Drop`
    lock: fs_lock::FileLock,
}

/// Builder for [`Locked`]
pub struct LockedBuilder {
    lock: fs_lock::FileLock,
}

impl LockedBuilder {
    /// Create a [`Self`] by acquiring a lock file
    pub async fn new(lock_path: &Path) -> anyhow::Result<LockedBuilder> {
        tokio::task::block_in_place(|| {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(lock_path)
                .with_context(|| format!("Failed to open {}", lock_path.display()))?;

            debug!(target: LOG_CLIENT, "Acquiring database lock");

            let lock = match fs_lock::FileLock::new_try_exclusive(file) {
                Ok(lock) => lock,
                Err((file, _)) => {
                    info!(target: LOG_CLIENT, "Waiting for the database lock");

                    fs_lock::FileLock::new_exclusive(file)
                        .context("Failed to acquire a lock file")?
                }
            };
            debug!(target: LOG_CLIENT, "Acquired database lock");

            Ok(LockedBuilder { lock })
        })
    }

    /// Create [`Locked`] by giving it the database to wrap
    pub fn with_db<DB>(self, db: DB) -> Locked<DB> {
        Locked {
            inner: db,
            lock: self.lock,
        }
    }
}

#[apply(async_trait_maybe_send!)]
impl<DB> IRawDatabase for Locked<DB>
where
    DB: IRawDatabase,
{
    type Transaction<'a> = DB::Transaction<'a>;

    async fn begin_transaction<'a>(
        &'a self,
    ) -> <Locked<DB> as fedimint_core::db::IRawDatabase>::Transaction<'_> {
        self.inner.begin_transaction().await
    }
}
// #### End of db_locked.rs from Fedimint CLI