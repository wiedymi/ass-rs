//! Tests for the multi-document session manager.

use super::*;
use crate::core::{EditorDocument, EditorError};
#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

#[test]
fn session_manager_creation() {
    let manager = EditorSessionManager::new();
    assert_eq!(manager.stats().active_sessions, 0);
    assert!(manager.active_session().unwrap().is_none());
}

#[test]
fn session_creation_and_switching() {
    let mut manager = EditorSessionManager::new();

    // Create first session
    manager.create_session("session1".to_string()).unwrap();
    assert_eq!(manager.stats().active_sessions, 1);
    assert_eq!(
        manager.active_session().unwrap(),
        Some("session1".to_string())
    );

    // Create second session
    manager.create_session("session2".to_string()).unwrap();
    assert_eq!(manager.stats().active_sessions, 2);

    // Switch to second session
    manager.switch_session("session2").unwrap();
    assert_eq!(
        manager.active_session().unwrap(),
        Some("session2".to_string())
    );

    // List sessions
    let sessions = manager.list_sessions().unwrap();
    assert_eq!(sessions.len(), 2);
    assert!(sessions.contains(&"session1".to_string()));
    assert!(sessions.contains(&"session2".to_string()));
}

#[test]
fn session_document_access() {
    let mut manager = EditorSessionManager::new();
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    manager
        .create_session_with_document("test".to_string(), doc)
        .unwrap();

    // Test document access
    manager
        .with_document("test", |doc| {
            assert!(doc.text().contains("Title: Test"));
            Ok(())
        })
        .unwrap();

    // Test document mutation
    manager
        .with_document_mut("test", |doc| {
            doc.insert(crate::core::Position::new(0), "Hello ")?;
            Ok(())
        })
        .unwrap();

    manager
        .with_document("test", |doc| {
            assert!(doc.text().starts_with("Hello "));
            Ok(())
        })
        .unwrap();
}

#[test]
fn session_removal() {
    let mut manager = EditorSessionManager::new();

    manager.create_session("test".to_string()).unwrap();
    assert_eq!(manager.stats().active_sessions, 1);

    let removed_session = manager.remove_session("test").unwrap();
    assert_eq!(removed_session.id, "test");
    assert_eq!(manager.stats().active_sessions, 0);
    assert!(manager.active_session().unwrap().is_none());
}

#[test]
fn session_limits() {
    let config = SessionConfig {
        max_sessions: 2,
        ..Default::default()
    };
    let mut manager = EditorSessionManager::with_config(config);

    // Create maximum allowed sessions
    manager.create_session("session1".to_string()).unwrap();
    manager.create_session("session2".to_string()).unwrap();

    // Try to create one more - should fail
    let result = manager.create_session("session3".to_string());
    assert!(matches!(
        result,
        Err(EditorError::SessionLimitExceeded { .. })
    ));
}

#[test]
fn session_metadata() {
    let mut session = EditorSession::new("test".to_string(), EditorDocument::new());

    assert_eq!(session.get_metadata("key"), None);

    session.set_metadata("key".to_string(), "value".to_string());
    assert_eq!(session.get_metadata("key"), Some("value"));
}
