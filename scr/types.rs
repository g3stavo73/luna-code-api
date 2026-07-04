
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DocumentId(pub(crate) u64);

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocumentId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub(crate) u64);

impl fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubscriptionId({})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub(crate) u64);

impl fmt::Display for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommandId({})", self.0)
    }
}

/// Posição dentro de um documento (linha e coluna, ambas com índice base 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    #[inline]
    pub fn new(line: usize, col: usize) -> Self { Self { line, col } }

    #[inline]
    pub fn origin() -> Self { Self { line: 0, col: 0 } }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}

/// Intervalo entre duas posições dentro de um documento.
/// `start` é inclusivo; `end` é exclusivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    #[inline]
    pub fn new(start: Position, end: Position) -> Self { Self { start, end } }

    #[inline]
    pub fn empty_at(pos: Position) -> Self { Self { start: pos, end: pos } }

    #[inline]
    pub fn is_empty(&self) -> bool { self.start == self.end }

    #[inline]
    pub fn is_single_line(&self) -> bool { self.start.line == self.end.line }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} → {}]", self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct DocumentInfo {
    pub id: DocumentId,
    pub path: Option<String>,
    pub display_name: String,
    pub is_dirty: bool,
    pub line_count: usize,
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub id: CommandId,
    pub name: String,
    pub description: String,
}
