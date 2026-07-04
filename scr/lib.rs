mod buffer;
mod document;

pub mod api;
pub mod commands;
pub mod errors;
pub mod events;
pub mod types;

pub use api::LunaApi;

pub use errors::LunaError;
pub use events::{Event, EventKind};

pub use types::{
    CommandId,
    CommandInfo,
    DocumentId,
    DocumentInfo,
    Position,
    Range,
    SubscriptionId,
};

pub mod prelude {
    pub use crate::{
        CommandId,
        CommandInfo,
        DocumentId,
        DocumentInfo,
        Event,
        EventKind,
        LunaApi,
        LunaError,
        Position,
        Range,
        SubscriptionId,
    };
}
