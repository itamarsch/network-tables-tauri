// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod communications;
use communications::{
    start_client, subscribe, unsubscribe, write, ConnectionState, PublishersState,
    SubscriptionsState, WritingCacheState,
};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,network_tables=debug")
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_client,
            subscribe,
            unsubscribe,
            write
        ])
        .manage(ConnectionState::default())
        .manage(SubscriptionsState::default())
        .manage(PublishersState::default())
        .manage(WritingCacheState::default())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
