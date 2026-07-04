//! Sistema de eventos do Luna Code.
//!
//! Callbacks são executados dentro de catch_unwind — um panic em um
//! subscriber nunca derruba a operação do editor nem afeta outros subscribers.

use std::{collections::HashMap, panic};
use crate::types::{DocumentId, Position, Range, SubscriptionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    FileOpened,
    FileSaved,
    FileClosed,
    CursorMoved,
    TextChanged,
}

#[derive(Debug, Clone)]
pub enum Event {
    FileOpened { doc_id: DocumentId, path: Option<String> },
    FileSaved { doc_id: DocumentId, path: String },
    FileClosed { doc_id: DocumentId },
    CursorMoved { doc_id: DocumentId, position: Position },
    TextChanged { doc_id: DocumentId, range: Range, new_text: String },
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::FileOpened { .. } => EventKind::FileOpened,
            Event::FileSaved { .. } => EventKind::FileSaved,
            Event::FileClosed { .. } => EventKind::FileClosed,
            Event::CursorMoved { .. } => EventKind::CursorMoved,
            Event::TextChanged { .. } => EventKind::TextChanged,
        }
    }
}

type Callback = Box<dyn Fn(&Event) + Send + Sync>;

pub(crate) struct EventBus {
    subscriptions: HashMap<EventKind, Vec<(SubscriptionId, Callback)>>,
    next_id: u64,
}

impl EventBus {
    pub fn new() -> Self {
        Self { subscriptions: HashMap::new(), next_id: 1 }
    }

    pub fn subscribe<F>(&mut self, kind: EventKind, callback: F) -> SubscriptionId
    where F: Fn(&Event) + Send + Sync + 'static {
        let id = SubscriptionId(self.next_id);
        self.next_id += 1;
        self.subscriptions.entry(kind).or_default().push((id, Box::new(callback)));
        id
    }

    pub fn unsubscribe(&mut self, id: SubscriptionId) {
        for subscribers in self.subscriptions.values_mut() {
            subscribers.retain(|(sub_id, _)| *sub_id != id);
        }
    }

    pub fn emit(&self, event: &Event) {
        if let Some(subscribers) = self.subscriptions.get(&event.kind()) {
            for (_, callback) in subscribers {
                let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| { callback(event); }));
            }
        }
    }
} 
