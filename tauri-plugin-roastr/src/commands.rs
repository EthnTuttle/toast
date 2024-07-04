use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use fedimint_client::{
    module::init::ClientModuleInitRegistry,
    secret::{PlainRootSecretStrategy, RootSecretStrategy},
    AdminCreds, Client,
};
use fedimint_core::{api::InviteCode, config::ClientConfig, module::ApiAuth, PeerId};
use roastr_client::{BroadcastEventResponse, RoastrClientInit, RoastrClientModule};
use roastr_common::{EventId, SignatureShare};
use tauri::{command, AppHandle, Runtime, State, Window};
use tracing::info;

use crate::{error::TauriPluginResult, MyState};

#[command]
pub(crate) async fn join_federation_as_admin(
    invite_code: String,
    admin_password: String,
    peer_id: String,
    state: State<'_, MyState>,
) -> TauriPluginResult<String> {
    let invite_code = InviteCode::from_str(&invite_code).expect("Bad invite code.");
    // TODO: determine what is returned for this. Does it have our peer ID? or perhaps multiple peer ids?
    // We have to know Which one we want to hit for auth reasons.
    let db = state.db.lock().unwrap().clone();
    let mut client_builder = Client::builder(db.clone());
    let mut client_module_registry = ClientModuleInitRegistry::new();
    client_module_registry.attach(RoastrClientInit);
    client_builder.with_module_inits(client_module_registry);
    client_builder.with_primary_module(0);
    let root_secret = Client::load_or_generate_client_secret(client_builder.db_no_decoders())
        .await
        .unwrap();
    let root_secret = PlainRootSecretStrategy::to_root_secret(&root_secret);
    let admin_creds = AdminCreds {
        peer_id: PeerId::from_str(&peer_id).expect("Bad peer id"),
        auth: ApiAuth(admin_password),
    };
    client_builder.set_admin_creds(admin_creds);
    let is_initialized = fedimint_client::Client::is_initialized(&db.clone().into()).await;
    if is_initialized {
        let client = client_builder
            .open(root_secret)
            .await
            .expect("Could not open client from db.");
        info!("done joining federation as admin");
        *state.client.lock().unwrap() = Some(Arc::new(client));
        Ok("success".to_string())
    } else {
        let config = ClientConfig::download_from_invite_code(&invite_code)
            .await
            .expect("Couldn't download config.");
        let client = client_builder
            .join(root_secret, config)
            .await
            .expect("Failed to join fedimint");
        // TODO: get config from the fedimint.
        info!("done joining federation as admin");
        *state.client.lock().unwrap() = Some(Arc::new(client));
        Ok("success".to_string())
    }
}

#[command]
pub(crate) async fn create_note<R: Runtime>(
    note_text: String,
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, MyState>,
) -> TauriPluginResult<EventId> {
    let cloned;
    {
        let client = state.client.lock().unwrap();
        cloned = client.clone();
    }
    let roastr = cloned.unwrap();
    let roastr_mod = roastr.get_first_module::<RoastrClientModule>();
    let event = roastr_mod.create_note(note_text).await.unwrap();
    Ok(event)
}

#[command]
pub(crate) async fn sign_note(
    event_id: EventId,
    state: State<'_, MyState>,
) -> TauriPluginResult<()> {
    let cloned;
    {
        let client = state.client.lock().unwrap();
        cloned = client.clone();
    }
    let client = cloned.unwrap();
    let roastr = client.get_first_module::<RoastrClientModule>();
    roastr.sign_note(event_id).await.unwrap();
    Ok(())
}

#[command]
pub(crate) async fn broadcast_note(
    event_id: EventId,
    state: State<'_, MyState>,
) -> TauriPluginResult<BroadcastEventResponse> {
    let cloned;
    {
        let client = state.client.lock().unwrap();
        cloned = client.clone();
    }
    let client = cloned.unwrap();
    let roastr = client.get_first_module::<RoastrClientModule>();
    let result = roastr.broadcast_note(event_id).await.unwrap();
    Ok(result)
}

#[command]
pub(crate) async fn get_signing_sessions(
    event_id: EventId,
    state: State<'_, MyState>
) -> TauriPluginResult<BTreeMap<String, BTreeMap<PeerId, SignatureShare>>> {
    let cloned;
    {
        let client = state.client.lock().unwrap();
        cloned = client.clone();
    }
    let client = cloned.unwrap();
    let roastr = client.get_first_module::<RoastrClientModule>();
    let result = roastr.get_signing_sessions(event_id).await.unwrap();
    Ok(result)
}
