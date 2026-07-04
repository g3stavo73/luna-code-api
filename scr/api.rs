use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    commands::CommandRegistry,
    document::Document,
    error::LunaError,
    events::{Event, EventBus, EventKind},
    types::{
        CommandId, CommandInfo, DocumentId, DocumentInfo, Position, Range, SubscriptionId,
    },
};

pub struct LunaApi {
    documents: HashMap<DocumentId, Document>,
    next_doc_id: u64,
    event_bus: EventBus,
    command_registry: CommandRegistry,
}

impl LunaApi {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            next_doc_id: 1,
            event_bus: EventBus::default(),
            command_registry: CommandRegistry::default(),
        }
    }

    fn next_id(&mut self) -> DocumentId {
        let id = DocumentId(self.next_doc_id);
        self.next_doc_id += 1;
        id
    }

    fn get_doc(&self, id: DocumentId) -> Result<&Document, LunaError> {
        self.documents
            .get(&id)
            .ok_or(LunaError::DocumentNotFound(id))
    }

    fn get_doc_mut(&mut self, id: DocumentId) -> Result<&mut Document, LunaError> {
        self.documents
            .get_mut(&id)
            .ok_or(LunaError::DocumentNotFound(id))
    }

    fn path_to_string(path: &Path) -> String {
        path.to_string_lossy().into_owned()
    }

    fn doc_path_to_string(doc: &Document) -> Option<String> {
        doc.path().map(Self::path_to_string)
    }

    fn canonicalize_path(path: &str) -> Result<PathBuf, LunaError> {
        std::fs::canonicalize(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                LunaError::FileNotFound(path.to_owned())
            } else {
                LunaError::io(path, e)
            }
        })
    }

    fn ensure_not_already_open(&self, canonical: &Path) -> Result<(), LunaError> {
        if self
            .documents
            .values()
            .any(|doc| matches!(doc.path(), Some(p) if p == canonical))
        {
            return Err(LunaError::DocumentAlreadyOpen(
                Self::path_to_string(canonical),
            ));
        }
        Ok(())
    }

    fn emit_file_opened(&self, doc_id: DocumentId, path: Option<String>) {
        self.event_bus.emit(&Event::FileOpened { doc_id, path });
    }

    fn emit_file_saved(&self, doc_id: DocumentId, path: String) {
        self.event_bus.emit(&Event::FileSaved { doc_id, path });
    }

    fn emit_file_closed(&self, doc_id: DocumentId) {
        self.event_bus.emit(&Event::FileClosed { doc_id });
    }

    fn emit_text_changed(&self, doc_id: DocumentId, range: Range, new_text: String) {
        self.event_bus.emit(&Event::TextChanged { doc_id, range, new_text });
    }

    fn emit_cursor_moved(&self, doc_id: DocumentId, position: Position) {
        self.event_bus.emit(&Event::CursorMoved { doc_id, position });
    }

    pub fn open_new(&mut self) -> DocumentId {
        let id = self.next_id();
        self.documents.insert(id, Document::new_empty(id));
        self.emit_file_opened(id, None);
        id
    }

    pub fn open_file(&mut self, path: &str) -> Result<DocumentId, LunaError> {
        let canonical = Self::canonicalize_path(path)?;
        self.ensure_not_already_open(&canonical)?;

        let content = std::fs::read_to_string(&canonical)
            .map_err(|e| LunaError::io(path, e))?;

        let id = self.next_id();
        let path_str = Self::path_to_string(&canonical);
        self.documents.insert(id, Document::from_disk(id, canonical, &content));

        self.emit_file_opened(id, Some(path_str));
        Ok(id)
    }

    pub fn save_file(&mut self, doc_id: DocumentId) -> Result<(), LunaError> {
        let path = {
            let doc = self.get_doc(doc_id)?;
            doc.path()
                .map(Self::path_to_string)
                .ok_or(LunaError::UnsavedDocument(doc_id))?
        };

        self.get_doc_mut(doc_id)?.save()?;
        self.emit_file_saved(doc_id, path);
        Ok(())
    }

    pub fn save_file_as(
        &mut self,
        doc_id: DocumentId,
        new_path: &str,
    ) -> Result<(), LunaError> {
        let doc = self.get_doc_mut(doc_id)?;
        doc.save_as(Path::new(new_path))?;
        let emitted_path =
            Self::doc_path_to_string(doc).unwrap_or_else(|| new_path.to_owned());

        self.emit_file_saved(doc_id, emitted_path);
        Ok(())
    }

    pub fn close_file(&mut self, doc_id: DocumentId) -> Result<(), LunaError> {
        if self.documents.remove(&doc_id).is_none() {
            return Err(LunaError::DocumentNotFound(doc_id));
        }
        self.emit_file_closed(doc_id);
        Ok(())
    }

    pub fn get_content(&self, doc_id: DocumentId) -> Result<String, LunaError> {
        Ok(self.get_doc(doc_id)?.content())
    }

    pub fn get_line(&self, doc_id: DocumentId, line: usize) -> Result<String, LunaError> {
        let doc = self.get_doc(doc_id)?;
        doc.get_line(line)
            .map(str::to_owned)
            .ok_or_else(|| {
                LunaError::invalid_pos(
                    doc_id,
                    Position::new(line, 0),
                    format!("documento tem {} linhas", doc.line_count()),
                )
            })
    }

    pub fn line_count(&self, doc_id: DocumentId) -> Result<usize, LunaError> {
        Ok(self.get_doc(doc_id)?.line_count())
    }

    pub fn is_dirty(&self, doc_id: DocumentId) -> Result<bool, LunaError> {
        Ok(self.get_doc(doc_id)?.is_dirty())
    }

    pub fn list_open_documents(&self) -> Vec<DocumentInfo> {
        let mut docs: Vec<DocumentInfo> =
            self.documents.values().map(Document::info).collect();
        docs.sort_unstable_by_key(|d| d.id.0);
        docs
    }

    pub fn insert_text(
        &mut self,
        doc_id: DocumentId,
        position: Position,
        text: &str,
    ) -> Result<(), LunaError> {
        let range = Range::empty_at(position);
        let new_text = text.to_owned();

        self.get_doc_mut(doc_id)?.insert_text(position, text)?;

        self.emit_text_changed(doc_id, range, new_text);
        Ok(())
    }

    pub fn delete_text(
        &mut self,
        doc_id: DocumentId,
        range: Range,
    ) -> Result<(), LunaError> {
        self.get_doc_mut(doc_id)?.delete_text(range)?;

        self.emit_text_changed(doc_id, range, String::new());
        Ok(())
    }

    pub fn replace_text(
        &mut self,
        doc_id: DocumentId,
        range: Range,
        new_text: &str,
    ) -> Result<(), LunaError> {
        let nt = new_text.to_owned();
        self.get_doc_mut(doc_id)?.replace_text(range, new_text)?;

        self.emit_text_changed(doc_id, range, nt);
        Ok(())
    }

    pub fn get_cursor(&self, doc_id: DocumentId) -> Result<Position, LunaError> {
        Ok(self.get_doc(doc_id)?.cursor())
    }

    pub fn set_cursor(
        &mut self,
        doc_id: DocumentId,
        position: Position,
    ) -> Result<(), LunaError> {
        self.get_doc_mut(doc_id)?.set_cursor(position)?;

        self.emit_cursor_moved(doc_id, position);
        Ok(())
    }

    pub fn subscribe<F>(&mut self, kind: EventKind, callback: F) -> SubscriptionId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.event_bus.subscribe(kind, callback)
    }

    pub fn unsubscribe(&mut self, subscription_id: SubscriptionId) {
        self.event_bus.unsubscribe(subscription_id);
    }

    pub fn register_command(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: impl Fn(&[String]) -> Result<Option<String>, LunaError> + Send + Sync + 'static,
    ) -> Result<CommandId, LunaError> {
        self.command_registry.register(name, description, handler)
    }

    pub fn unregister_command(&mut self, name: &str) {
        self.command_registry.unregister(name);
    }

    pub fn execute_command(
        &self,
        name: &str,
        args: &[String],
    ) -> Result<Option<String>, LunaError> {
        self.command_registry.execute(name, args)
    }

    pub fn list_commands(&self) -> Vec<CommandInfo> {
        self.command_registry.list()
    }

    pub fn has_command(&self, name: &str) -> bool {
        self.command_registry.contains(name)
    }
}

impl Default for LunaApi {
    fn default() -> Self {
        Self::new()
    }
        }
