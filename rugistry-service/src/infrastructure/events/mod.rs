use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

use crate::application::dto::ChangeNotification;

#[derive(Clone)]
pub struct EventBus {
    sender: Arc<Mutex<broadcast::Sender<ChangeNotification>>>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(Mutex::new(sender)),
        }
    }

    pub fn publish(&self, event: ChangeNotification) {
        let sender = self.sender.lock().unwrap();
        let _ = sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ChangeNotification> {
        let sender = self.sender.lock().unwrap();
        sender.subscribe()
    }
}
