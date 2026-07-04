#![forbid(unsafe_code)]

mod buffer;
mod document;

pub(crate) mod commands;
pub(crate) mod events;
pub(crate) mod error;
pub(crate) mod types;

mod api;

pub use api::LunaApi;

pub use types::{
    CommandId,
    CommandInfo,
    DocumentId,
    DocumentInfo,
    Position,
    Range,
    SubscriptionId,
};

pub use events::{Event, EventKind};

pub use error::LunaError;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn open_new_document_and_read_empty_content() {
        let mut api = LunaApi::new();
        let doc = api.open_new();
        assert_eq!(api.get_content(doc).unwrap(), "");
        assert_eq!(api.line_count(doc).unwrap(), 1);
        assert!(!api.is_dirty(doc).unwrap());
    }

    #[test]
    fn open_multiple_documents_independently() {
        let mut api = LunaApi::new();
        let doc1 = api.open_new();
        let doc2 = api.open_new();

        api.insert_text(doc1, Position::origin(), "conteúdo 1").unwrap();
        api.insert_text(doc2, Position::origin(), "conteúdo 2").unwrap();

        assert_eq!(api.get_content(doc1).unwrap(), "conteúdo 1");
        assert_eq!(api.get_content(doc2).unwrap(), "conteúdo 2");
    }

    #[test]
    fn close_document_makes_id_invalid() {
        let mut api = LunaApi::new();
        let doc = api.open_new();
        api.close_file(doc).unwrap();

        let result = api.get_content(doc);
        assert!(matches!(result, Err(LunaError::DocumentNotFound(_))));
    }

    #[test]
    fn list_open_documents() {
        let mut api = LunaApi::new();
        assert!(api.list_open_documents().is_empty());

        let doc1 = api.open_new();
        let doc2 = api.open_new();

        let docs = api.list_open_documents();
        assert_eq!(docs.len(), 2);
        assert!(docs.iter().any(|d| d.id == doc1));
        assert!(docs.iter().any(|d| d.id == doc2));
    }

    #[test]
    fn insert_text_marks_document_dirty() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        assert!(!api.is_dirty(doc).unwrap());
        api.insert_text(doc, Position::origin(), "x").unwrap();
        assert!(api.is_dirty(doc).unwrap());
    }

    #[test]
    fn insert_text_multiline() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        api.insert_text(doc, Position::origin(), "linha 1\nlinha 2\nlinha 3").unwrap();

        assert_eq!(api.line_count(doc).unwrap(), 3);
        assert_eq!(api.get_line(doc, 0).unwrap(), "linha 1");
        assert_eq!(api.get_line(doc, 1).unwrap(), "linha 2");
        assert_eq!(api.get_line(doc, 2).unwrap(), "linha 3");
    }

    #[test]
    fn insert_at_middle_of_line() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        api.insert_text(doc, Position::origin(), "helo").unwrap();
        api.insert_text(doc, Position::new(0, 3), "l").unwrap();

        assert_eq!(api.get_content(doc).unwrap(), "hello");
    }

    #[test]
    fn delete_text_single_line() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        api.insert_text(doc, Position::origin(), "hello world").unwrap();
        let range = Range::new(Position::new(0, 5), Position::new(0, 11));
        api.delete_text(doc, range).unwrap();

        assert_eq!(api.get_content(doc).unwrap(), "hello");
    }

    #[test]
    fn delete_text_multiline() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        api.insert_text(doc, Position::origin(), "linha 1\nlinha 2\nlinha 3").unwrap();
        let range = Range::new(Position::new(0, 7), Position::new(1, 7));
        api.delete_text(doc, range).unwrap();

        assert_eq!(api.line_count(doc).unwrap(), 2);
        assert_eq!(api.get_content(doc).unwrap(), "linha 1\nlinha 3");
    }

    #[test]
    fn replace_text() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        api.insert_text(doc, Position::origin(), "foo bar baz").unwrap();
        let range = Range::new(Position::new(0, 4), Position::new(0, 7));
        api.replace_text(doc, range, "luna").unwrap();

        assert_eq!(api.get_content(doc).unwrap(), "foo luna baz");
    }

    #[test]
    fn cursor_starts_at_origin() {
        let mut api = LunaApi::new();
        let doc = api.open_new();
        assert_eq!(api.get_cursor(doc).unwrap(), Position::origin());
    }

    #[test]
    fn set_cursor_valid_position() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        api.insert_text(doc, Position::origin(), "abc\ndef").unwrap();
        api.set_cursor(doc, Position::new(1, 2)).unwrap();

        assert_eq!(api.get_cursor(doc).unwrap(), Position::new(1, 2));
    }

    #[test]
    fn set_cursor_invalid_position_returns_error() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        let result = api.set_cursor(doc, Position::new(99, 0));
        assert!(matches!(result, Err(LunaError::InvalidPosition { .. })));
    }

    #[test]
    fn file_opened_event_emitted_on_open_new() {
        let mut api = LunaApi::new();
        let fired: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        let fired_clone = fired.clone();
        api.subscribe(EventKind::FileOpened, move |_| {
            *fired_clone.lock().unwrap() = true;
        });

        api.open_new();
        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn text_changed_event_carries_correct_new_text() {
        let mut api = LunaApi::new();
        let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let cap = captured.clone();
        api.subscribe(EventKind::TextChanged, move |event| {
            if let Event::TextChanged { new_text, .. } = event {
                *cap.lock().unwrap() = Some(new_text.clone());
            }
        });

        let doc = api.open_new();
        api.insert_text(doc, Position::origin(), "luna code").unwrap();

        assert_eq!(*captured.lock().unwrap(), Some("luna code".to_owned()));
    }

    #[test]
    fn cursor_moved_event_emitted_on_set_cursor() {
        let mut api = LunaApi::new();
        let fired: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        let f = fired.clone();
        api.subscribe(EventKind::CursorMoved, move |_| {
            *f.lock().unwrap() = true;
        });

        let doc = api.open_new();
        api.insert_text(doc, Position::origin(), "abc").unwrap();
        api.set_cursor(doc, Position::new(0, 2)).unwrap();

        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn file_closed_event_emitted() {
        let mut api = LunaApi::new();
        let fired: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        let f = fired.clone();
        api.subscribe(EventKind::FileClosed, move |_| {
            *f.lock().unwrap() = true;
        });

        let doc = api.open_new();
        api.close_file(doc).unwrap();

        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn unsubscribe_stops_receiving_events() {
        let mut api = LunaApi::new();
        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

        let c = count.clone();
        let sub_id = api.subscribe(EventKind::TextChanged, move |_| {
            *c.lock().unwrap() += 1;
        });

        let doc = api.open_new();
        api.insert_text(doc, Position::origin(), "a").unwrap();
        assert_eq!(*count.lock().unwrap(), 1);

        api.unsubscribe(sub_id);
        api.insert_text(doc, Position::new(0, 1), "b").unwrap();
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn register_and_execute_command() {
        let mut api = LunaApi::new();

        api.register_command("test.ping", "Verifica se a API responde", |_args| {
            Ok(Some("pong".to_owned()))
        }).unwrap();

        let result = api.execute_command("test.ping", &[]).unwrap();
        assert_eq!(result, Some("pong".to_owned()));
    }

    #[test]
    fn duplicate_command_registration_returns_specific_error() {
        let mut api = LunaApi::new();
        api.register_command("cmd.x", "Primeiro", |_| Ok(None)).unwrap();
        let err = api.register_command("cmd.x", "Duplicado", |_| Ok(None)).unwrap_err();
        assert!(
            matches!(err, LunaError::CommandAlreadyRegistered(_)),
            "esperava CommandAlreadyRegistered, recebeu: {err:?}"
        );
    }

    #[test]
    fn unregister_command_frees_name() {
        let mut api = LunaApi::new();
        api.register_command("cmd.y", "Original", |_| Ok(None)).unwrap();
        api.unregister_command("cmd.y");
        api.register_command("cmd.y", "Novo", |_| Ok(None)).unwrap();
    }

    #[test]
    fn execute_unknown_command_returns_error() {
        let api = LunaApi::new();
        assert!(matches!(
            api.execute_command("nao.existe", &[]),
            Err(LunaError::CommandNotFound(_))
        ));
    }

    #[test]
    fn list_commands_includes_registered() {
        let mut api = LunaApi::new();
        api.register_command("file.save", "Salva o arquivo", |_| Ok(None)).unwrap();
        api.register_command("format.doc", "Formata o documento", |_| Ok(None)).unwrap();

        let cmds = api.list_commands();
        assert_eq!(cmds.len(), 2);
        assert!(cmds.iter().any(|c| c.name == "file.save"));
        assert!(cmds.iter().any(|c| c.name == "format.doc"));
    }

    #[test]
    fn has_command_reflects_registration() {
        let mut api = LunaApi::new();
        assert!(!api.has_command("x.cmd"));
        api.register_command("x.cmd", "X", |_| Ok(None)).unwrap();
        assert!(api.has_command("x.cmd"));
        api.unregister_command("x.cmd");
        assert!(!api.has_command("x.cmd"));
    }

    #[test]
    fn operations_on_invalid_doc_id_return_error() {
        let mut api = LunaApi::new();
        let fake_id = DocumentId(9999);

        assert!(matches!(
            api.get_content(fake_id),
            Err(LunaError::DocumentNotFound(_))
        ));
        assert!(matches!(
            api.insert_text(fake_id, Position::origin(), "x"),
            Err(LunaError::DocumentNotFound(_))
        ));
        assert!(matches!(
            api.close_file(fake_id),
            Err(LunaError::DocumentNotFound(_))
        ));
    }

    #[test]
    fn save_unsaved_document_returns_correct_error() {
        let mut api = LunaApi::new();
        let doc = api.open_new();

        let err = api.save_file(doc).unwrap_err();
        assert!(
            matches!(err, LunaError::UnsavedDocument(_)),
            "esperava UnsavedDocument, recebeu: {err:?}"
        );
    }
                   }
