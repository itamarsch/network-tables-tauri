use std::collections::HashMap;

use network_tables::v4 as nt;
use tauri::async_runtime::{JoinHandle, Mutex};
use tauri::{Manager, State};

#[derive(Default)]
pub struct SubscriptionsState(Mutex<HashMap<String, u16>>);

impl SubscriptionsState {
    async fn is_subscribed_to(&self, topic: &str) -> bool {
        self.0.lock().await.contains_key(topic)
    }
}

const SUBSCRIPTION_OPTIONS: nt::subscription::SubscriptionOptions =
    nt::subscription::SubscriptionOptions {
        // https://github.com/wpilibsuite/allwpilib/blob/main/ntcore/doc/networktables4.adoc#sub-options
        // Dont realy know what rest means
        prefix: Some(true),
        all: Some(true),
        periodic: None,
        topics_only: None,
        rest: None,
    };

#[tauri::command]
pub async fn subscribe(
    topic: String,
    subscription_state: State<'_, SubscriptionsState>,
) -> Result<(), ()> {
    let mut subscriptions = subscription_state.0.lock().await;
    match subscriptions.get_mut(&topic) {
        Some(value) => {
            *value += 1;
        }
        None => {
            subscriptions.insert(topic.clone(), 1);
        }
    }

    tracing::trace!(
        "Subscribing to {:?}, {:?}",
        topic,
        subscriptions.get(&topic).unwrap()
    );
    Ok(())
}

#[tauri::command]
pub async fn unsubscribe(
    topic: String,
    subscription_state: State<'_, SubscriptionsState>,
) -> Result<(), String> {
    let mut subscriptions = subscription_state.0.lock().await;

    let Some(amount_of_subscriptions) = subscriptions.get(&topic) else {
        return Err(String::from("Not subscribed to topic"));
    };

    if *amount_of_subscriptions <= 1 {
        subscriptions.remove_entry(&topic);
    } else {
        *subscriptions.get_mut(&topic).unwrap() -= 1;
    }

    tracing::trace!(
        "Unsubscribing from {:?}, {:?}",
        topic,
        subscriptions.get(&topic)
    );
    Ok(())
}

pub async fn subscribe_task(client: nt::Client, window: tauri::Window) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        loop {
            let subscription = client
                .subscribe_w_options(&[""], Some(SUBSCRIPTION_OPTIONS))
                .await;

            let mut subscription = match subscription {
                Ok(sub) => sub,
                Err(err) => {
                    tracing::error!("Subscibing failed {:?}", err);
                    continue;
                }
            };

            // TODO: test ignoring 'None' and removing 'loop'
            while let Some(message) = subscription.next().await {
                let state = window.state::<SubscriptionsState>();

                let is_topic_subscribed = state.is_subscribed_to(&message.topic_name).await;

                if is_topic_subscribed {
                    tracing::trace!("Received Data from {:?}", message.topic_name,);
                    window.emit(&message.topic_name.clone(), message).ok();
                }
            }

            if let Err(error) = client.unsubscribe(subscription).await {
                tracing::error!("error unsubscribing {:?}", error);
            };

            tracing::warn!("While loop ended subscribing again");
        }
    })
}
