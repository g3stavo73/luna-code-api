use crate::types::{DocumentId, Position, Range};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LunaError {
    #[error("documento não encontrado: {0}")]
    DocumentNotFound(DocumentId),

    #[error("arquivo já aberto: '{0}'")]
    DocumentAlreadyOpen(String),

    #[error("documento {0} não tem caminho definido; use save_file_as")]
    UnsavedDocument(DocumentId),

    #[error("posição inválida {position} no documento {doc_id} ({detail})")]
    InvalidPosition {
        doc_id: DocumentId,
        position: Position,
        detail: String,
    },

    #[error("range inválido {range} no documento {doc_id} ({detail})")]
    InvalidRange {
        doc_id: DocumentId,
        range: Range,
        detail: String,
    },

    #[error("arquivo não encontrado: '{0}'")]
    FileNotFound(String),

    #[error("erro de I/O em '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("comando não registrado: '{0}'")]
    CommandNotFound(String),

    #[error("comando '{0}' já está registrado; cancele o registro anterior ou use um nome diferente")]
    CommandAlreadyRegistered(String),

    #[error("falha ao executar o comando '{name}': {reason}")]
    CommandExecutionFailed { name: String, reason: String },

    #[error("erro interno: {0}")]
    Internal(String),
}

impl LunaError {
    pub(crate) fn io(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::IoError { path: path.into(), source }
    }

    pub(crate) fn invalid_pos(
        doc_id: DocumentId,
        position: Position,
        detail: impl Into<String>,
    ) -> Self {
        Self::InvalidPosition { doc_id, position, detail: detail.into() }
    }

    pub(crate) fn invalid_range(
        doc_id: DocumentId,
        range: Range,
        detail: impl Into<String>,
    ) -> Self {
        Self::InvalidRange { doc_id, range, detail: detail.into() }
    }
}
