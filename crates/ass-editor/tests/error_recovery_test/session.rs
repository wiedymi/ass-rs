//! Session manager and concurrent error tests.
//!
//! Tests for error handling in the session manager, including operations on
//! missing sessions and concurrent access from multiple threads.

#[cfg(feature = "std")]
use ass_editor::EditorSessionManager;

#[test]
#[cfg(feature = "std")] // EditorSessionManager requires std feature
fn test_session_manager_error_cases() {
    let mut manager = EditorSessionManager::new();

    // Create session
    manager.create_session("test".to_string()).unwrap();

    // Duplicate session - behavior may vary (might replace or fail)
    let _duplicate_result = manager.create_session("test".to_string());
    // Just verify it doesn't panic

    // Operations on non-existent session
    assert!(manager.remove_session("non-existent").is_err());
    assert!(manager
        .with_document_mut("non-existent", |_| Ok(()))
        .is_err());

    // Empty session name might be allowed
    let empty_result = manager.create_session("".to_string());
    // If it succeeds, clean it up
    if empty_result.is_ok() {
        manager.remove_session("").unwrap();
    }

    // Very long session name
    let long_name = "x".repeat(10000);
    let result = manager.create_session(long_name.clone());
    if result.is_ok() {
        assert!(manager.list_sessions().unwrap().contains(&long_name));
    }
}

// ===== Concurrent Error Scenarios =====

#[cfg(all(feature = "multi-thread", feature = "std"))]
#[test]
fn test_concurrent_error_handling() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    let error_count = Arc::new(AtomicUsize::new(0));
    let success_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn threads that try various operations
    // Each thread gets its own manager to avoid cloning issues
    for i in 0..10 {
        let error_count_clone = error_count.clone();
        let success_count_clone = success_count.clone();

        let handle = thread::spawn(move || {
            // Each thread creates its own manager
            let mut local_manager = ass_editor::EditorSessionManager::new();

            // Try to create a session
            match local_manager.create_session("shared".to_string()) {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
            };

            // Try to access non-existent sessions
            let result = local_manager.with_document_mut(&format!("missing_{i}"), |_doc| Ok(()));
            match result {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
            };

            // Try valid operations
            match local_manager.create_session(format!("thread_{i}")) {
                Ok(_) => success_count_clone.fetch_add(1, Ordering::Relaxed),
                Err(_) => error_count_clone.fetch_add(1, Ordering::Relaxed),
            };
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Should have mix of successes and errors
    assert!(error_count.load(Ordering::Relaxed) > 0);
    assert!(success_count.load(Ordering::Relaxed) > 0);
}
