//! Integration tests for ass-editor components
//!
//! These tests are designed to work regardless of features enabled.
//! Feature-specific functionality is tested only when the feature is available.

use ass_editor::*;

#[test]
fn test_basic_document_operations() {
    // These should always work
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    // Basic operations
    doc.insert(Position::new(doc.len_bytes()), "\nAuthor: Test")
        .unwrap();
    assert!(doc.text().contains("Author: Test"));

    // Undo/redo
    doc.undo().unwrap();
    assert!(!doc.text().contains("Author: Test"));
    doc.redo().unwrap();
    assert!(doc.text().contains("Author: Test"));
}

#[test]
fn test_basic_editing() {
    let mut doc = EditorDocument::new();

    // Insert
    doc.insert(Position::new(0), "Hello World").unwrap();
    assert_eq!(doc.text(), "Hello World");

    // Delete
    doc.delete(Range::new(Position::new(0), Position::new(5)))
        .unwrap();
    assert_eq!(doc.text(), " World");

    // Replace
    doc.replace(Range::new(Position::new(0), Position::new(6)), "Goodbye")
        .unwrap();
    assert_eq!(doc.text(), "Goodbye");
}

#[test]
fn test_extension_manager_basic() {
    let manager = ExtensionManager::new();
    let extensions = manager.list_extensions();
    assert_eq!(extensions.len(), 0);
}

// Advanced tests that require specific features
#[cfg(all(feature = "std", feature = "multi-thread", feature = "plugins"))]
mod advanced_tests {
    use super::*;
    use ass_editor::events::DocumentEvent;
    use std::sync::mpsc;
    use std::thread;
    // Thread-safe managers are now unified - no need for separate imports

    #[test]
    fn test_extension_manager_with_document_integration() {
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello"
        ).unwrap();

        let mut context = manager
            .create_context("test_extension".to_string(), Some(&mut doc))
            .unwrap();

        context
            .show_message("Test message", MessageLevel::Info)
            .unwrap();
        context
            .set_config("test_key".to_string(), "test_value".to_string())
            .unwrap();
        assert_eq!(
            context.get_config("test_key"),
            Some("test_value".to_string())
        );

        assert!(context.current_document().is_some());
        if let Some(doc) = context.current_document_mut() {
            doc.insert(Position::new(0), "# ").unwrap();
        }
    }

    #[test]
    fn test_session_manager_with_extensions_integration() {
        let mut session_manager = EditorSessionManager::new();

        session_manager
            .create_session("session1".to_string())
            .unwrap();
        session_manager
            .create_session("session2".to_string())
            .unwrap();

        session_manager
            .with_document_mut("session1", |doc| {
                doc.replace(
                    Range::new(Position::new(0), Position::new(0)),
                    "[Script Info]\nTitle: Session 1",
                )
            })
            .unwrap();

        session_manager
            .with_document_mut("session2", |doc| {
                doc.replace(
                    Range::new(Position::new(0), Position::new(0)),
                    "[Script Info]\nTitle: Session 2",
                )
            })
            .unwrap();

        session_manager
            .with_document("session1", |doc| {
                assert!(doc.text().contains("Session 1"));
                assert!(!doc.text().contains("Session 2"));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn test_thread_safe_extension_manager() {
        let manager = ExtensionManager::new();
        let manager1 = manager.clone();
        let manager2 = manager.clone();

        let handle1 = thread::spawn(move || {
            let mut manager_mut = manager1;
            manager_mut.set_config("thread1_key".to_string(), "thread1_value".to_string());
        });

        let handle2 = thread::spawn(move || {
            let mut manager_mut = manager2;
            manager_mut.set_config("thread2_key".to_string(), "thread2_value".to_string());
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // The unified manager is already thread-safe internally
        assert_eq!(
            manager.get_config("thread1_key"),
            Some("thread1_value".to_string())
        );
        assert_eq!(
            manager.get_config("thread2_key"),
            Some("thread2_value".to_string())
        );
    }

    #[test]
    fn test_thread_safe_session_manager() {
        let manager = EditorSessionManager::new();
        let manager1 = manager.clone();
        let manager2 = manager.clone();

        let handle1 = thread::spawn(move || {
            let mut manager_mut = manager1;
            manager_mut
                .create_session("thread1_session".to_string())
                .unwrap();
        });

        let handle2 = thread::spawn(move || {
            let mut manager_mut = manager2;
            manager_mut
                .create_session("thread2_session".to_string())
                .unwrap();
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // The unified manager is already thread-safe internally
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.contains(&"thread1_session".to_string()));
        assert!(sessions.contains(&"thread2_session".to_string()));
    }

    #[test]
    fn test_document_events_integration() {
        let (tx, rx) = mpsc::channel();
        let mut doc = EditorDocument::with_event_channel(tx);

        doc.insert(Position::new(0), "Hello").unwrap();
        doc.delete(Range::new(Position::new(0), Position::new(5)))
            .unwrap();
        doc.replace(Range::new(Position::new(0), Position::new(0)), "World")
            .unwrap();

        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            events.push(event);
        }

        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], DocumentEvent::TextInserted { .. }));
        assert!(matches!(events[1], DocumentEvent::TextDeleted { .. }));
        assert!(matches!(events[2], DocumentEvent::TextReplaced { .. }));
    }

    #[test]
    fn test_extension_loading_and_state() {
        let mut manager = ExtensionManager::new();

        manager
            .load_extension(Box::new(
                ass_editor::extensions::builtin::syntax_highlight::SyntaxHighlightExtension::new(),
            ))
            .unwrap();

        manager
            .load_extension(Box::new(
                ass_editor::extensions::builtin::auto_complete::AutoCompleteExtension::new(),
            ))
            .unwrap();

        let extensions = manager.list_extensions();
        assert!(extensions.contains(&"syntax-highlight".to_string()));
        assert!(extensions.contains(&"auto-complete".to_string()));

        assert_eq!(
            manager.get_extension_state("syntax-highlight"),
            Some(ExtensionState::Uninitialized)
        );
    }
}
