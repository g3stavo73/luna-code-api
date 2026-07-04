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
    FileOpened {
        doc_id: DocumentId,
        path: Option<String>,
    },
    FileSaved {
        doc_id: DocumentId,
        path: String,
    },
    FileClosed {
        doc_id: DocumentId,
    },
    CursorMoved {
        doc_id: DocumentId,
        position: Position,
    },
    TextChanged {
        doc_id: DocumentId,
        range: Range,
        new_text: String,
    },
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Self::FileOpened { .. }  => EventKind::FileOpened,
            Self::FileSaved { .. }   => EventKind::FileSaved,
            Self::FileClosed { .. }  => EventKind::FileClosed,
            Self::CursorMoved { .. } => EventKind::CursorMoved,
            Self::TextChanged { .. } => EventKind::TextChanged,
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
        Self {
            subscriptions: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn subscribe<F>(&mut self, kind: EventKind, callback: F) -> SubscriptionId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let id = SubscriptionId(self.next_id);
        self.next_id += 1;
        self.subscriptions
            .entry(kind)
            .or_default()
            .push((id, Box::new(callback)));
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
                let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    callback(event);
                }));
            }
        }
    }

    #[cfg(test)]
    pub fn subscription_count(&self, kind: EventKind) -> usize {
        self.subscriptions
            .get(&kind)
            .map_or(0, Vec::len)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DocumentId, Position, Range};
    use std::sync::{Arc, Mutex};

    #[test]
    fn subscribe_and_receive_event() {
        let mut bus = EventBus::new();
        let received: Arc<Mutex<Vec<EventKind>>> = Arc::new(Mutex::new(vec![]));

        let received_clone = received.clone();
        bus.subscribe(EventKind::FileSaved, move |e| {
            received_clone.lock().unwrap().push(e.kind());
        });

        bus.emit(&Event::FileSaved {
            doc_id: DocumentId(1),
            path: "/tmp/test.rs".to_owned(),
        });

        let events = received.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], EventKind::FileSaved);
    }

    #[test]
    fn unsubscribe_stops_callbacks() {
        let mut bus = EventBus::new();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

        let count_clone = count.clone();
        let sub_id = bus.subscribe(EventKind::CursorMoved, move |_| {
            *count_clone.lock().unwrap() += 1;
        });

        let event = Event::CursorMoved {
            doc_id: DocumentId(1),
            position: Position::new(0, 0),
        };

        bus.emit(&event);
        assert_eq!(*count.lock().unwrap(), 1);

        bus.unsubscribe(sub_id);
        bus.emit(&event);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn different_event_kinds_are_isolated() {
        let mut bus = EventBus::new();
        let received: Arc<Mutex<Vec<EventKind>>> = Arc::new(Mutex::new(vec![]));

        let r = received.clone();
        bus.subscribe(EventKind::FileOpened, move |e| {
            r.lock().unwrap().push(e.kind());
        });

        bus.emit(&Event::FileSaved {
            doc_id: DocumentId(1),
            path: "/tmp/x.rs".to_owned(),
        });

        assert!(received.lock().unwrap().is_empty());
    }

    #[test]
    fn text_changed_event_carries_data() {
        let mut bus = EventBus::new();
        let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let cap = captured.clone();
        bus.subscribe(EventKind::TextChanged, move |e| {
            if let Event::TextChanged { new_text, .. } = e {
                *cap.lock().unwrap() = Some(new_text.clone());
            }
        });

        bus.emit(&Event::TextChanged {
            doc_id: DocumentId(1),
            range: Range::empty_at(Position::new(0, 0)),
            new_text: "olá".to_owned(),
        });

        assert_eq!(*captured.lock().unwrap(), Some("olá".to_owned()));
    }

    #[test]
    fn panicking_subscriber_does_not_propagate() {
        let mut bus = EventBus::new();
        let reached: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        bus.subscribe(EventKind::FileSaved, |_| {
            panic!("subscriber com bug");
        });

        let r = reached.clone();
        bus.subscribe(EventKind::FileSaved, move |_| {
            *r.lock().unwrap() = true;
        });

        bus.emit(&Event::FileSaved {
            doc_id: DocumentId(1),
            path: "/tmp/x.rs".to_owned(),
        });

        assert!(*reached.lock().unwrap());
    }

    #[test]
    fn event_bus_default_is_empty() {
        let bus = EventBus::default();
        assert_eq!(bus.subscription_count(EventKind::FileOpened), 0);
    }
                                                            }    pub fn to_text(&self) -> String { self.lines.join("\n") }

    pub fn is_valid_position(&self, pos: Position) -> bool {
        match self.lines.get(pos.line) {
            None => false,
            Some(line) => pos.col <= line.len() && line.is_char_boundary(pos.col),
        }
    }

    pub fn is_valid_range(&self, range: Range) -> bool {
        if !self.is_valid_position(range.start) || !self.is_valid_position(range.end) {
            return false;
        }
        (range.start.line, range.start.col) <= (range.end.line, range.end.col)
    }

    pub fn insert(&mut self, doc_id: DocumentId, pos: Position, text: &str) -> Result<(), LunaError> {
        if !self.is_valid_position(pos) {
            return Err(LunaError::invalid_pos(doc_id, pos,
                format!("documento tem {} linhas; linha {} tem {} bytes",
                    self.lines.len(), pos.line,
                    self.lines.get(pos.line).map(|l| l.len()).unwrap_or(0))));
        }
        let prefix = self.lines[pos.line][..pos.col].to_owned();
        let suffix = self.lines[pos.line][pos.col..].to_owned();
        let new_parts: Vec<&str> = text.split('\n').collect();
        if new_parts.len() == 1 {
            self.lines[pos.line] = format!("{}{}{}", prefix, new_parts[0], suffix);
        } else {
            let first = format!("{}{}", prefix, new_parts[0]);
            let last = format!("{}{}", new_parts[new_parts.len() - 1], suffix);
            let mut replacement = vec![first];
            for part in &new_parts[1..new_parts.len() - 1] { replacement.push(part.to_string()); }
            replacement.push(last);
            let tail = self.lines.split_off(pos.line + 1);
            self.lines.pop();
            self.lines.extend(replacement);
            self.lines.extend(tail);
        }
        Ok(())
    }

    pub fn delete(&mut self, doc_id: DocumentId, range: Range) -> Result<(), LunaError> {
        if !self.is_valid_range(range) {
            return Err(LunaError::invalid_range(doc_id, range,
                "range fora dos limites do buffer ou em fronteira UTF-8 inválida"));
        }
        if range.is_empty() { return Ok(()); }
        let Range { start, end } = range;
        if start.line == end.line {
            self.lines[start.line].replace_range(start.col..end.col, "");
        } else {
            let prefix = self.lines[start.line][..start.col].to_owned();
            let suffix = self.lines[end.line][end.col..].to_owned();
            let merged = format!("{}{}", prefix, suffix);
            self.lines.splice(start.line..=end.line, [merged]);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn replace(&mut self, doc_id: DocumentId, range: Range, new_text: &str) -> Result<(), LunaError> {
        self.delete(doc_id, range)?;
        self.insert(doc_id, range.start, new_text)
    }

    pub fn adjust_cursor_after_insert(
        cursor: Position, insert_pos: Position,
        inserted_lines: usize, last_col_delta: usize,
    ) -> Position {
        if cursor.line < insert_pos.line { return cursor; }
        if cursor.line == insert_pos.line {
            if cursor.col < insert_pos.col { return cursor; }
            if inserted_lines == 0 {
                return Position::new(cursor.line, cursor.col + last_col_delta);
            } else {
                let new_line = cursor.line + inserted_lines;
                let new_col = last_col_delta + (cursor.col - insert_pos.col);
                return Position::new(new_line, new_col);
            }
        }
        Position::new(cursor.line + inserted_lines, cursor.col)
    }

    pub fn adjust_cursor_after_delete(cursor: Position, range: Range) -> Position {
        if range.is_empty() { return cursor; }
        let (start, end) = (range.start, range.end);
        if (cursor.line, cursor.col) <= (start.line, start.col) { return cursor; }
        if (cursor.line, cursor.col) <= (end.line, end.col) { return start; }
        let deleted_lines = end.line - start.line;
        if cursor.line == end.line {
            Position::new(start.line, start.col + (cursor.col - end.col))
        } else {
            Position::new(cursor.line - deleted_lines, cursor.col)
        }
    }
}

impl Default for Buffer { fn default() -> Self { Self::new() } } 
