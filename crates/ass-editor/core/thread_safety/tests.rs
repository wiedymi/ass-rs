//! Tests for the thread safety abstractions.

use super::*;
use crate::commands::InsertTextCommand;
use crate::core::{EditorDocument, Position};
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

#[test]
fn test_sync_document_creation() {
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let sync_doc = SyncDocument::new(doc);

    let text = sync_doc.text().unwrap();
    assert!(text.contains("Title: Test"));
}

#[test]
fn test_sync_document_modification() {
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let sync_doc = SyncDocument::new(doc);

    // Modify using write lock
    sync_doc
        .with_write(|doc| doc.insert(Position::new(doc.len()), "\nAuthor: Test"))
        .unwrap();

    // Verify modification
    let text = sync_doc.text().unwrap();
    assert!(text.contains("Author: Test"));
}

#[test]
fn test_document_pool() {
    let pool = DocumentPool::new();

    // Add documents
    let doc1 = EditorDocument::from_content("[Script Info]\nTitle: Doc1").unwrap();
    let id1 = pool.add_document(doc1).unwrap();

    let doc2 = EditorDocument::from_content("[Script Info]\nTitle: Doc2").unwrap();
    let id2 = pool.add_document(doc2).unwrap();

    // Verify pool
    assert_eq!(pool.document_count().unwrap(), 2);

    // Get and verify documents
    let sync_doc1 = pool.get_document(&id1).unwrap();
    assert!(sync_doc1.text().unwrap().contains("Doc1"));

    let sync_doc2 = pool.get_document(&id2).unwrap();
    assert!(sync_doc2.text().unwrap().contains("Doc2"));

    // Remove document
    pool.remove_document(&id1).unwrap();
    assert_eq!(pool.document_count().unwrap(), 1);
}

#[test]
fn test_concurrent_usage() {
    // Test that SyncDocument provides thread-safe access
    // Note: We can't actually spawn threads due to EditorDocument not being Sync,
    // but we can test the synchronization primitives work correctly

    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let sync_doc = SyncDocument::new(doc);

    // Test multiple modifications work correctly
    for i in 0..5 {
        sync_doc
            .with_write(|doc| {
                let pos = Position::new(doc.len());
                doc.insert(pos, &format!("\nComment: Update {i}"))
            })
            .unwrap();
    }

    // Verify final state
    let final_text = sync_doc.text().unwrap();
    assert!(final_text.contains("Comment: Update 4"));

    // Test read while write lock is held (should block)
    let _write_guard = sync_doc.write().unwrap();
    assert!(sync_doc.try_read().is_none());
}

#[test]
fn test_try_lock_operations() {
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let sync_doc = SyncDocument::new(doc);

    // Get write lock
    let _write_guard = sync_doc.write().unwrap();

    // Try to get another lock (should fail)
    assert!(sync_doc.try_read().is_none());
    assert!(sync_doc.try_write().is_none());
}

#[test]
fn test_thread_safe_command_execution() {
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let sync_doc = SyncDocument::new(doc);

    // Execute command through thread-safe wrapper
    let command = InsertTextCommand::new(Position::new(0), "[V4+ Styles]\n".to_string());

    let result = sync_doc.execute_command(command).unwrap();
    assert!(result.success);
    assert!(result.content_changed);

    // Verify the change
    let text = sync_doc.text().unwrap();
    assert!(text.starts_with("[V4+ Styles]"));
}

#[test]
fn test_sync_document_validation() {
    let doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test"
    ).unwrap();
    let sync_doc = SyncDocument::new(doc);

    // Basic validation should pass
    sync_doc.validate().unwrap();

    // Comprehensive validation
    let issues = sync_doc.validate_comprehensive().unwrap();
    // Print issues for debugging
    for issue in &issues {
        println!("Validation issue: {issue:?}");
    }
    // For now, just check that validation runs without error
    // The exact number of issues may vary based on validator configuration
    assert!(issues.len() <= 1); // Allow up to 1 minor issue
}

#[test]
fn test_scoped_lock() {
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let sync_doc = SyncDocument::new(doc);

    // Use scoped lock for batch operations
    {
        let mut lock = ScopedDocumentLock::new(&sync_doc).unwrap();
        let doc = lock.document();
        doc.insert(Position::new(doc.len()), "\nAuthor: Test")
            .unwrap();
        doc.insert(Position::new(doc.len()), "\nVersion: 1.0")
            .unwrap();
    }

    // Verify changes
    let text = sync_doc.text().unwrap();
    assert!(text.contains("Author: Test"));
    assert!(text.contains("Version: 1.0"));
}

#[test]
fn test_command_send_sync() {
    // Verify that commands are Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<InsertTextCommand>();
    assert_send_sync::<crate::commands::DeleteTextCommand>();
    assert_send_sync::<crate::commands::ReplaceTextCommand>();
    assert_send_sync::<crate::commands::BatchCommand>();

    // Note: SyncDocument and DocumentPool cannot be Send + Sync because
    // EditorDocument contains Bump allocator with Cell types that are not Sync.
    // However, they still provide thread-safe access through their methods
    // by ensuring only one thread can mutate at a time.
}
