use directories::ProjectDirs;
use fedimint_client::{module::init::ClientModuleInitRegistry, Client, ClientBuilder, ClientHandle, ClientHandleArc};
use fedimint_core::{
    api::InviteCode, apply, async_trait_maybe_send, config::FederationId, db::Database,
    util::SafeUrl, PeerId,
};
use roastr_client::RoastrClientInit;
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Debug, sync::Mutex};
use tauri::{
    generate_handler, plugin::{Builder, TauriPlugin}, Manager, Runtime
};
use crate::commands::{create_note, join_federation_as_admin};

use core::fmt;
use std::{path::PathBuf, result};
use thiserror::Error;

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

#[cfg(desktop)]
use desktop::Roastr;
#[cfg(mobile)]
use mobile::Roastr;

struct MyState {
    db: Mutex<Database>,
    // probably need a vec of clients? or a hashmap?
    client: Mutex<Option<ClientHandle>>,
    our_peer_id: Mutex<Option<PeerId>>,
    password: Mutex<Option<String>>,
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
        // .invoke_handler(tauri::generate_handler![commands::execute])
        .setup(|app, api| {
            #[cfg(mobile)]
            let roastr = mobile::init(app, api)?;
            #[cfg(desktop)]
            let roastr = desktop::init(app, api)?;
            app.manage(roastr);

            let db = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(load_rocks_db())
                .expect("Failed to load rocks db");
 
            // manage state so it is accessible by the commands
            app.manage(MyState {
                db: db.into(),
                client: None.into(),
                our_peer_id: None.into(),
                password: None.into(),
            });
            Ok(())
        })
        .invoke_handler(generate_handler![create_note, join_federation_as_admin])
        .build()
}

async fn load_rocks_db() -> CliResult<Database> {
    debug!(target: LOG_CLIENT, "Loading client database");
    let db_path = data_dir_create().await?.join("client.db");
    let lock_path = db_path.with_extension("db.lock");
    Ok(LockedBuilder::new(&lock_path)
        .await
        .map_err_cli_msg("could not lock database")?
        .with_db(
            fedimint_rocksdb::RocksDb::open(db_path).map_err_cli_msg("could not open database")?,
        )
        .into())
}

fn data_dir() -> CliResult<PathBuf> {
    let dirs = ProjectDirs::from(
        "org",         /*qualifier*/
        "Baz Corp",    /*organization*/
        "Foo Bar-App", /*application*/
    )
    .expect("Could not create data dir")
    .data_dir()
    .to_owned();
    let path_buf = PathBuf::from(dirs);
    Ok(path_buf)
}

/// Get and create if doesn't exist the data dir
async fn data_dir_create() -> CliResult<PathBuf> {
    let dir = data_dir()?;

    tokio::fs::create_dir_all(&dir).await.map_err_cli()?;

    Ok(dir)
}

/// Type of output the cli produces
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum CliOutput {
    VersionHash {
        hash: String,
    },

    UntypedApiOutput {
        value: Value,
    },

    WaitBlockCount {
        reached: u64,
    },

    InviteCode {
        invite_code: InviteCode,
    },

    DecodeInviteCode {
        url: SafeUrl,
        federation_id: FederationId,
    },

    JoinFederation {
        joined: String,
    },

    DecodeTransaction {
        transaction: String,
    },

    EpochCount {
        count: u64,
    },

    ConfigDecrypt,

    ConfigEncrypt,

    Raw(serde_json::Value),
}

/// `Result` with `CliError` as `Error`
type CliResult<E> = Result<E, CliError>;

/// `Result` with `CliError` as `Error` and `CliOutput` as `Ok`
type CliOutputResult = Result<CliOutput, CliError>;

/// Cli error
#[derive(Serialize, Error)]
#[serde(tag = "error", rename_all(serialize = "snake_case"))]
struct CliError {
    error: String,
}

impl Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliError")
            .field("error", &self.error)
            .finish()
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json = serde_json::to_value(self).expect("CliError is valid json");
        let json_as_string =
            serde_json::to_string_pretty(&json).expect("valid json is serializable");
        write!(f, "{}", json_as_string)
    }
}

/// Extension trait making turning Results/Errors into
/// [`CliError`]/[`CliOutputResult`] easier
trait CliResultExt<O, E> {
    /// Map error into `CliError` wrapping the original error message
    fn map_err_cli(self) -> Result<O, CliError>;
    /// Map error into `CliError` using custom error message `msg`
    fn map_err_cli_msg(self, msg: impl Into<String>) -> Result<O, CliError>;
}

impl<O, E> CliResultExt<O, E> for result::Result<O, E>
where
    E: Into<anyhow::Error>,
{
    fn map_err_cli(self) -> Result<O, CliError> {
        self.map_err(|e| {
            let e = e.into();
            CliError {
                error: e.to_string(),
            }
        })
    }

    fn map_err_cli_msg(self, msg: impl Into<String>) -> Result<O, CliError> {
        self.map_err(|_| CliError { error: msg.into() })
    }
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
