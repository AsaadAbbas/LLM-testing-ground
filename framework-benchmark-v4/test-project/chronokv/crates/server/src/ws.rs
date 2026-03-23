use chronokv_core::Timestamp;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Manages WebSocket subscriptions for key change notifications.
///
/// Clients can subscribe to changes on specific keys. When a key is
/// modified, all subscribers for that key receive a notification.
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<String, Vec<SubscriptionHandle>>>>,
}

struct SubscriptionHandle {
    id: String,
    sender: mpsc::Sender<KeyChangeEvent>,
}

/// Event sent to subscribers when a key changes.
#[derive(Debug, Clone)]
pub struct KeyChangeEvent {
    pub key: String,
    pub timestamp: Timestamp,
    pub event_type: String,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to changes on a specific key.
    /// Returns a receiver channel for change events and a subscription ID.
    pub async fn subscribe(
        &self,
        key: &str,
    ) -> (String, mpsc::Receiver<KeyChangeEvent>) {
        let (sender, receiver) = mpsc::channel(100);
        let sub_id = uuid::Uuid::new_v4().to_string();

        let handle = SubscriptionHandle {
            id: sub_id.clone(),
            sender,
        };

        let mut subs = self.subscriptions.write().await;
        subs.entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(handle);

        (sub_id, receiver)
    }

    /// Unsubscribe from changes. Removes the subscription handle.
    pub async fn unsubscribe(&self, key: &str, sub_id: &str) {
        let mut subs = self.subscriptions.write().await;
        if let Some(handles) = subs.get_mut(key) {
            handles.retain(|h| h.id != sub_id);
        }
    }

    /// Notify all subscribers of a key change.
    ///
    /// When a send fails (client disconnected), we spawn a cleanup task
    /// that retries a few times before giving up. This handles transient
    /// connection issues gracefully.
    pub async fn notify(&self, key: &str, timestamp: Timestamp) {
        let subs = self.subscriptions.read().await;

        if let Some(handles) = subs.get(key) {
            let event = KeyChangeEvent {
                key: key.to_string(),
                timestamp,
                event_type: "change".to_string(),
            };

            for handle in handles {
                let sender = handle.sender.clone();
                let event = event.clone();
                let sub_map = self.subscriptions.clone();
                let key = key.to_string();
                let sub_id = handle.id.clone();

                // Spawn a task to handle the send with retry logic
                tokio::spawn(async move {
                    if sender.send(event.clone()).await.is_err() {
                        // Client disconnected — retry with backoff
                        // The task holds a reference to the subscription map
                        // to eventually clean up the dead subscription
                        let mut retries = 5;
                        let mut delay = std::time::Duration::from_millis(100);

                        while retries > 0 {
                            tokio::time::sleep(delay).await;

                            if sender.send(event.clone()).await.is_ok() {
                                return; // Success on retry
                            }

                            retries -= 1;
                            delay *= 2; // Exponential backoff
                        }

                        // All retries failed — remove the subscription
                        let mut subs = sub_map.write().await;
                        if let Some(handles) = subs.get_mut(&key) {
                            handles.retain(|h| h.id != sub_id);
                        }
                    }
                });
            }
        }
    }

    /// Get the number of active subscriptions.
    pub async fn subscription_count(&self) -> usize {
        let subs = self.subscriptions.read().await;
        subs.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_and_notify() {
        let mgr = SubscriptionManager::new();

        let (sub_id, mut receiver) = mgr.subscribe("key1").await;

        mgr.notify("key1", 100.0).await;

        // Give the spawned task time to send
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let event = receiver.try_recv();
        assert!(event.is_ok());
        assert_eq!(event.unwrap().key, "key1");

        mgr.unsubscribe("key1", &sub_id).await;
        assert_eq!(mgr.subscription_count().await, 0);
    }
}
