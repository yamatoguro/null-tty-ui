use std::collections::VecDeque;

/// Known event topics across the UI system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Topic {
    /// A plugin panel content changed.
    PanelUpdate { region: String },
}

/// A typed message delivered across bus channels.
#[derive(Debug, Clone)]
pub struct Event {
    pub topic: Topic,
    pub payload: Option<String>,
}

impl Event {
    /// Creates an event carrying an associated string payload.
    pub fn with_payload(topic: Topic, payload: impl Into<String>) -> Self {
        Self {
            topic,
            payload: Some(payload.into()),
        }
    }
}

/// A lightweight in-process pub/sub bus backed by a bounded ring-buffer queue.
/// Capped to max_capacity events so slow consumers cannot cause unbounded growth.
pub struct EventBus {
    queue: VecDeque<Event>,
    max_capacity: usize,
}

impl EventBus {
    /// Creates a bus with the given event drop threshold.
    pub fn new(max_capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    /// Pushes an event onto the bus, dropping the oldest if at capacity.
    pub fn publish(&mut self, event: Event) {
        if self.queue.len() >= self.max_capacity {
            self.queue.pop_front();
        }
        self.queue.push_back(event);
    }

    /// Drains all pending events and returns them oldest-first.
    pub fn drain(&mut self) -> Vec<Event> {
        self.queue.drain(..).collect()
    }
}
