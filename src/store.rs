use std::collections::{LinkedList, HashMap};
use std::time::{Instant, Duration};
use async_std::stream::{self, StreamExt};
use async_std::sync::Mutex;

type Message = Vec<u8>;

#[derive(Clone)]
pub struct Store {
    queues: std::sync::Arc<Mutex<HashMap<String, Queue>>>,
}

pub struct Queue {
    messages: LinkedList<Message>,
}

impl Queue {
    fn new() -> Self {
        Queue {
            messages: LinkedList::new()
        }
    }

    fn push(&mut self, message: Message) {
        self.messages.push_back(message)
    }

    fn pop(&mut self) -> Option<Message> {
        self.messages.pop_front()
    }
}

impl Store {
    pub fn new() -> Self {
        Store {
            queues: std::sync::Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn pop_sync(&self, key: &str) -> Option<Message> {
        self.queues
            .lock()
            .await
            .get_mut(key)
            .map(|q| q.pop())
            .flatten()
    }

    pub async fn push(&self, key: &str, msg: Message) {
        self.queues
            .lock()
            .await
            .entry(key.to_string())
            .or_insert_with(Queue::new)
            .push(msg)
    }

    pub async fn pop(&self, key: &str, timeout: Option<Duration>) -> Option<Message> {
        match timeout {
            None => {
                self.pop_sync(key).await
            }
            Some(timeout) => {
                let deadline = Instant::now() + timeout;
                let mut ticker = stream::interval(Duration::from_millis(10)).
                    take_while(|_| Instant::now() < deadline);
                while ticker.next().await.is_some() {
                    match deadline.checked_duration_since(Instant::now()) {
                        Some(pop_timeout) => {
                            if let Some(msg) = async_std::future::timeout(
                                pop_timeout,
                                self.pop_sync(key),
                            )
                                .await
                                .ok()
                                .flatten()
                            {
                                return Some(msg);
                            }
                        }
                        None => return None
                    }
                }
                return None;
            }
        }
    }
}
