use network_tables::v4 as nt;
use std::collections::HashMap;
use tauri::async_runtime::Mutex;
use tauri::State;

use super::ConnectionState;

#[derive(Default)]
pub struct WritingCacheState(pub Mutex<HashMap<String, network_tables::Value>>);

#[derive(Default)]
pub struct PublishersState(pub Mutex<HashMap<String, nt::PublishedTopic>>);

pub async fn flush_cache(
    client: &nt::Client,
    cache: &mut HashMap<String, network_tables::Value>,
    publishers: &mut HashMap<String, nt::PublishedTopic>,
) -> Result<(), String> {
    tracing::info!("Writing cache {:?}", cache);
    for (topic, value) in cache.iter() {
        publish_value(&topic, &value, value_to_type(value)?, client, publishers).await?;
    }
    cache.clear();
    tracing::info!("Writing cache {:?}", cache);
    Ok(())
}

#[tauri::command]
pub async fn write(
    topic: String,
    value: network_tables::Value,
    client: State<'_, ConnectionState>,
    publishers: State<'_, PublishersState>,
    cache: State<'_, WritingCacheState>,
    window: tauri::Window,
) -> Result<(), String> {
    let client_and_handle = client.0.lock().await;
    let mut publishers = publishers.0.lock().await;
    let mut cache = cache.0.lock().await;
    let value = nt_number_to_f64(value); // Converting to double if only it's a number
    let nt_type = value_to_type(&value)?;

    window
        .emit(
            &topic,
            nt::MessageData {
                timestamp: 0,
                data: value.clone(),
                topic_name: topic.clone(),
                r#type: nt_type,
            },
        )
        .map_err(|err| err.to_string())?;

    let Some((client,_)) = client_and_handle.as_ref() else {
        tracing::trace!("Cache writing {:?} to {:?}", value, &topic);
        cache.insert(topic, value);
        return Ok(());
    };
    publish_value(&topic, &value, nt_type, client, &mut publishers).await
}

async fn publish_value(
    topic: &str,
    value: &network_tables::Value,
    nt_type: nt::Type,
    client: &nt::Client,
    publishers: &mut HashMap<String, nt::PublishedTopic>,
) -> Result<(), String> {
    // Saving publishers for topics.
    // If a publisher exists for this topic then use it to write the value.
    // If not, create one, save it, and write the value.
    if !publishers.contains_key(topic) {
        let published_topic = client
            .publish_topic(&topic, nt_type, None)
            .await
            .map_err(|e| e.to_string())?;
        publishers.insert(String::from(topic), published_topic);
    };

    let publisher = publishers.get(topic).unwrap();
    tracing::debug!("Writing {:?} to {:?}", value, &topic);
    client
        .publish_value(publisher, &value)
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn value_to_type(value: &network_tables::Value) -> Result<nt::Type, String> {
    match value {
        network_tables::Value::F64(_) => Ok(nt::Type::Double),
        network_tables::Value::String(_) => Ok(nt::Type::String),
        network_tables::Value::Boolean(_) => Ok(nt::Type::Boolean),
        _ => Err(String::from("Not a valid nt Type")),
    }
}

fn nt_number_to_f64(value: network_tables::Value) -> network_tables::Value {
    if value.is_number() {
        network_tables::Value::F64(value.as_f64().unwrap())
    } else {
        value
    }
}
