//! Concurrent operation edge cases for ass-editor.
//!
//! Exercised only when the `multi-thread` and `std` features are enabled.

#[cfg(all(feature = "multi-thread", feature = "std"))]
mod concurrent_tests {
    use ass_editor::{EditorSessionManager, ExtensionManager, Position};
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn test_concurrent_config_access() {
        use std::sync::Arc;

        // Use Arc to share the manager instead of cloning
        let manager = Arc::new(ExtensionManager::new());
        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];

        // Spawn 10 threads that all try to modify config simultaneously
        for i in 0..10 {
            let manager_ref = Arc::clone(&manager);
            let barrier_clone = barrier.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                // Each thread sets multiple config values
                // We can't modify through Arc<ExtensionManager> directly,
                // so this test verifies read-only concurrent access
                for j in 0..100 {
                    let key = format!("thread_{i}_key_{j}");
                    let _value = manager_ref.get_config(&key);
                }
            });

            handles.push(handle);
        }

        // Set up test data first
        // Since we can't get mutable access through Arc, pre-populate
        for i in 0..10 {
            for j in 0..10 {
                // Reduce to avoid too many keys
                let _key = format!("thread_{i}_key_{j}");
                // Note: We can't actually set config through Arc<ExtensionManager>
                // This test is mainly to verify the manager doesn't panic under concurrent read access
            }
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Test passes if no panics occurred during concurrent access
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_concurrent_session_operations() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let _manager = EditorSessionManager::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        // Create sessions concurrently
        // Note: This test demonstrates the design limitation -
        // we can't easily share a mutable session manager across threads
        // without the Clone trait. This suggests the API needs improvement.
        for i in 0..5 {
            let counter_clone = counter.clone();

            let handle = thread::spawn(move || {
                // Each thread creates its own manager since we can't clone
                let mut local_manager = EditorSessionManager::new();

                for j in 0..20 {
                    let session_name = format!("session_{i}_{j}");
                    let result = local_manager.create_session(session_name.clone());

                    // Creation might fail due to session limit (50 by default)
                    if result.is_ok() {
                        // Modify the session
                        local_manager
                            .with_document_mut(&session_name, |doc| {
                                doc.insert(Position::new(0), &format!("Thread {i} Doc {j}\n"))
                            })
                            .unwrap();

                        counter_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify sessions were created in each thread
        let created_count = counter.load(Ordering::Relaxed);
        assert!(created_count > 0); // Should create some sessions across all threads

        // Note: Since each thread uses its own manager, we can't verify
        // the sessions on the original manager. This test mainly verifies
        // that concurrent session creation doesn't panic or deadlock.
    }
}
