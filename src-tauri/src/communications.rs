use std::{net::SocketAddr, str::FromStr};

use network_tables::v4 as nt;
use tauri::{
    async_runtime::{JoinHandle, Mutex},
    State,
};

mod subscribe;
pub use subscribe::{subscribe, unsubscribe, SubscriptionsState};

mod write;
pub use write::{write, PublishersState, WritingCacheState};

#[derive(Default)]
pub struct ConnectionState(pub Mutex<Option<(nt::Client, JoinHandle<()>)>>);

const CONNECT_TIMEOUT_MILLIS: u64 = 3000;

#[tauri::command]
pub async fn start_client(
    ip: &str,
    connection_state: State<'_, ConnectionState>,
    publisher_state: State<'_, PublishersState>,
    cache: State<'_, WritingCacheState>,
    window: tauri::Window,
) -> Result<(), String> {
    let client = connect(ip).await?;

    tracing::info!("Connected to client");
    window
        .emit("Connect-client", true)
        .map_err(|err| err.to_string())?;

    let mut client_and_handle = connection_state.0.lock().await;
    if let Some((_, handle)) = client_and_handle.as_ref() {
        handle.abort(); // terminate previous subscription task
    }

    let task_client = client.clone();
    let handle = subscribe::subscribe_task(task_client, window).await;

    let mut publishers = publisher_state.0.lock().await;
    publishers.clear(); // clear from previous connection

    let mut cache = cache.0.lock().await;
    write::flush_cache(&client, &mut cache, &mut publishers).await?;

    *client_and_handle = Some((client, handle));
    Ok(())
}

async fn connect(ip: &str) -> Result<nt::Client, String> {
    let client = nt::Client::try_new_w_config(
        SocketAddr::from_str(ip).map_err(|_| "InvalidIp")?,
        nt::Config {
            connect_timeout: CONNECT_TIMEOUT_MILLIS,
            ..Default::default()
        },
    )
    .await
    .map_err(|err| err.to_string())?;

    Ok(client)
}
