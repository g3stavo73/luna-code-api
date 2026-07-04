//! Representação de um documento aberto no Luna Code.
//!
//! O cursor é sempre mantido em uma posição válida. Após qualquer mutação
//! de texto, o cursor é ajustado automaticamente.

use std::path::{Path, PathBuf};
use crate::{buffer::Buffer, error::LunaError, types::{DocumentId, DocumentInfo, Position, Range}};

#[derive(Debug)]
pub struct Document {
    pub(crate) id: DocumentId,
    pub(crate) path: Option<PathBuf>,
    pub(crate) buffer: Buffer,
    pub(crate) cursor: Position,
    pub(crate) is_dirty: bool,
}

impl Document {
    pub(crate) fn new_empty(id: DocumentId) -> Self {
        Self { id, path: None, buffer: Buffer::new(), cursor: Position::origin(), is_dirty: false }
    }

    pub(crate) fn from_disk(id: DocumentId, path: PathBuf, content: &str) -> Self {
        Self { id, path: Some(path), buffer: Buffer::from_text(content), cursor: Position::origin(), is_dirty: false }
    }

    pub fn path(&self) -> Option<&Path> { self.path.as_deref() }
    pub fn is_dirty(&self) -> bool { self.is_dirty }
    pub fn cursor(&self) -> Position { self.cursor }
    pub fn line_count(&self) -> usize { self.buffer.line_count() }
    pub fn content(&self) -> String { self.buffer.to_text() }
    pub fn get_line(&self, line: usize) -> Option<&str> { self.buffer.get_line(line) }

    pub fn display_name(&self) -> String {
        self.path.as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Sem título".to_owned())
    }

    pub fn info(&self) -> DocumentInfo {
        DocumentInfo {
            id: self.id,
            path: self.path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            display_name: self.display_name(),
            is_dirty: self.is_dirty,
            line_count: self.buffer.line_count(),
        }
    }

    pub(crate) fn insert_text(&mut self, pos: Position, text: &str) -> Result<(), LunaError> {
        let parts: Vec<&str> = text.split('\n').collect();
        let inserted_lines = parts.len() - 1;
        let last_col_delta = parts.last().map(|s| s.len()).unwrap_or(0);
        self.buffer.insert(self.id, pos, text)?;
        self.cursor = Buffer::adjust_cursor_after_insert(self.cursor, pos, inserted_lines, last_col_delta);
        self.is_dirty = true;
        Ok(())
    }

    pub(crate) fn delete_text(&mut self, range: Range) -> Result<(), LunaError> {
        self.buffer.delete(self.id, range)?;
        self.cursor = Buffer::adjust_cursor_after_delete(self.cursor, range);
        self.is_dirty = true;
        Ok(())
    }

    pub(crate) fn replace_text(&mut self, range: Range, new_text: &str) -> Result<(), LunaError> {
        self.delete_text(range)?;
        self.insert_text(range.start, new_text)
    }

    pub(crate) fn set_cursor(&mut self, pos: Position) -> Result<(), LunaError> {
        if !self.buffer.is_valid_position(pos) {
            return Err(LunaError::invalid_pos(self.id, pos,
                format!("documento tem {} linhas", self.buffer.line_count())));
        }
        self.cursor = pos;
        Ok(())
    }

    pub(crate) fn save(&mut self) -> Result<(), LunaError> {
        let path = self.path.clone().ok_or_else(|| LunaError::Internal(
            format!("documento {} não tem caminho definido; use save_file_as", self.id)))?;
        std::fs::write(&path, self.buffer.to_text())
            .map_err(|e| LunaError::io(path.to_string_lossy(), e))?;
        self.is_dirty = false;
        Ok(())
    }

    pub(crate) fn save_as(&mut self, new_path: &Path) -> Result<(), LunaError> {
        std::fs::write(new_path, self.buffer.to_text())
            .map_err(|e| LunaError::io(new_path.to_string_lossy(), e))?;
        self.path = Some(new_path.to_owned());
        self.is_dirty = false;
        Ok(())
    }
} 
