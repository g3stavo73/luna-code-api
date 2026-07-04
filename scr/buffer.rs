//! Buffer de texto do Luna Code.
//!
//! O Buffer armazena o conteúdo de um documento como um vetor de linhas
//! (Vec<String>). Todos os offsets de coluna são validados contra fronteiras
//! de char UTF-8 antes de qualquer slice.

use crate::{error::LunaError, types::{DocumentId, Position, Range}};

#[derive(Debug, Clone)]
pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self { Self { lines: vec![String::new()] } }

    pub fn from_text(text: &str) -> Self {
        if text.is_empty() { return Self::new(); }
        let lines: Vec<String> = text.lines().map(|l| l.to_owned()).collect();
        if text.ends_with('\n') {
            let mut buf = Self { lines };
            buf.lines.push(String::new());
            buf
        } else {
            Self { lines }
        }
    }

    #[inline]
    pub fn line_count(&self) -> usize { self.lines.len() }

    #[inline]
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.lines.get(line).map(|s| s.as_str())
    }

    pub fn to_text(&self) -> String { self.lines.join("\n") }

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
