//! Extended tests for auto-completion extension

#[cfg(test)]
mod tests {
    use crate::core::{EditorDocument, Position};
    use crate::extensions::builtin::auto_complete::{
        AutoCompleteExtension, CompletionContext, CompletionItem, CompletionType,
    };
    use crate::extensions::{EditorExtension, ExtensionManager, ExtensionState};
    use std::collections::HashMap;

    #[test]
    fn test_section_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "[Scr".to_string(),
            column: 4,
            section: None,
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_section_completions(&context);

        // Should have Script Info completion
        assert!(!completions.is_empty());
        let script_info = completions
            .iter()
            .find(|c| c.label == "[Script Info]")
            .unwrap();
        assert_eq!(script_info.completion_type, CompletionType::Section);
        assert!(script_info.description.is_some());
    }

    #[test]
    fn test_empty_line_section_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "".to_string(),
            column: 0,
            section: None,
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_section_completions(&context);

        // Should show all sections
        assert_eq!(completions.len(), 6);
        assert!(completions.iter().any(|c| c.label == "[Script Info]"));
        assert!(completions.iter().any(|c| c.label == "[V4+ Styles]"));
        assert!(completions.iter().any(|c| c.label == "[Events]"));
    }

    #[test]
    fn test_script_info_field_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "Ti".to_string(),
            column: 2,
            section: Some("Script Info".to_string()),
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_field_completions("Script Info", &context);

        // Should have Title completion
        let title = completions.iter().find(|c| c.label == "Title:").unwrap();
        assert_eq!(title.completion_type, CompletionType::Field);
    }

    #[test]
    fn test_playres_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "PlayRes".to_string(),
            column: 7,
            section: Some("Script Info".to_string()),
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_field_completions("Script Info", &context);

        // Should have both PlayResX and PlayResY
        assert!(completions.iter().any(|c| c.label == "PlayResX:"));
        assert!(completions.iter().any(|c| c.label == "PlayResY:"));
    }

    #[test]
    fn test_events_field_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "".to_string(),
            column: 0,
            section: Some("Events".to_string()),
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_field_completions("Events", &context);

        // Should have all event types
        assert!(completions.iter().any(|c| c.label == "Dialogue:"));
        assert!(completions.iter().any(|c| c.label == "Comment:"));
        assert!(completions.iter().any(|c| c.label == "Format:"));
    }

    #[test]
    fn test_override_tag_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "{\\b".to_string(),
            column: 3,
            section: Some("Events".to_string()),
            in_override_tag: true,
            current_tag: Some("b".to_string()),
        };

        let completions = ext.get_tag_completions(&context);

        // Should have bold tag
        let bold = completions.iter().find(|c| c.label == "\\b").unwrap();
        assert_eq!(bold.completion_type, CompletionType::Tag);
        assert_eq!(bold.insert_text, "\\b1");
    }

    #[test]
    fn test_tag_completions_with_prefix() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "{\\po".to_string(),
            column: 4,
            section: Some("Events".to_string()),
            in_override_tag: true,
            current_tag: Some("po".to_string()),
        };

        let completions = ext.get_tag_completions(&context);

        // Should have pos tag
        let pos = completions.iter().find(|c| c.label == "\\pos").unwrap();
        assert_eq!(pos.insert_text, "\\pos(640,360)");
    }

    #[test]
    fn test_color_tag_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "{\\".to_string(),
            column: 2,
            section: Some("Events".to_string()),
            in_override_tag: true,
            current_tag: None,
        };

        let completions = ext.get_tag_completions(&context);

        // Should have color tags
        assert!(completions.iter().any(|c| c.label == "\\c"));
        assert!(completions.iter().any(|c| c.label == "\\1c"));
        assert!(completions.iter().any(|c| c.label == "\\2c"));
        assert!(completions.iter().any(|c| c.label == "\\3c"));
        assert!(completions.iter().any(|c| c.label == "\\4c"));
    }

    #[test]
    fn test_style_completions() {
        let mut ext = AutoCompleteExtension::new();
        let doc = EditorDocument::from_content(
            "[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1\nStyle: Title,Times New Roman,30,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,",
        )
        .unwrap();

        // Update style names
        ext.update_style_names(&doc).unwrap();

        let context = CompletionContext {
            line: "Dialogue: 0,0:00:00.00,0:00:05.00,".to_string(),
            column: 34,
            section: Some("Events".to_string()),
            in_override_tag: false,
            current_tag: None,
        };

        // Check if we should complete styles
        assert!(ext.should_complete_style(&context));

        let completions = ext.get_style_completions(&context);

        // Debug output if test fails
        if completions.is_empty() {
            println!(
                "No style completions found. Style names: {:?}",
                ext.style_names
            );
        }

        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.insert_text == "Default"));
        assert!(completions.iter().any(|c| c.insert_text == "Title"));
    }

    #[test]
    fn test_completion_context_parsing() {
        let ext = AutoCompleteExtension::new();
        let doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Test\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}text",
        )
        .unwrap();

        // Test context at different positions
        let context1 = ext.get_completion_context(&doc, Position::new(25)).unwrap(); // After Title:
        assert_eq!(context1.section, Some("Script Info".to_string()));
        assert!(!context1.in_override_tag);

        // In override tag
        let tag_pos = doc.text().find("{\\b").unwrap() + 2;
        let context2 = ext
            .get_completion_context(&doc, Position::new(tag_pos))
            .unwrap();
        assert!(context2.in_override_tag);
    }

    #[test]
    fn test_get_completions_integration() {
        let mut ext = AutoCompleteExtension::new();
        let doc = EditorDocument::from_content("[Script Info]\nTi").unwrap();

        let completions = ext
            .get_completions(&doc, Position::new(doc.len_bytes()))
            .unwrap();

        // Should have Title completion
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.label == "Title:"));
    }

    #[test]
    fn test_extension_lifecycle() {
        let mut ext = AutoCompleteExtension::new();
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::new();
        let mut context = manager
            .create_context("test".to_string(), Some(&mut doc))
            .unwrap();

        // Initialize
        assert_eq!(ext.state(), ExtensionState::Uninitialized);
        ext.initialize(&mut *context).unwrap();
        assert_eq!(ext.state(), ExtensionState::Active);

        // Execute trigger command
        let mut args = HashMap::new();
        args.insert("position".to_string(), "0".to_string());
        let result = ext
            .execute_command("autocomplete.trigger", &args, &mut *context)
            .unwrap();
        assert!(result.success);

        // Shutdown
        ext.shutdown(&mut *context).unwrap();
        assert_eq!(ext.state(), ExtensionState::Shutdown);
    }

    #[test]
    fn test_config_loading() {
        let mut ext = AutoCompleteExtension::new();
        let mut manager = ExtensionManager::new();

        // Set configuration
        manager.set_config(
            "autocomplete.complete_fields".to_string(),
            "false".to_string(),
        );
        manager.set_config("autocomplete.max_suggestions".to_string(), "10".to_string());

        let mut doc = EditorDocument::new();
        let mut context = manager
            .create_context("test".to_string(), Some(&mut doc))
            .unwrap();

        // Initialize should load config
        ext.initialize(&mut *context).unwrap();

        // Config should be loaded
        assert!(!ext.config.complete_fields);
        assert_eq!(ext.config.max_suggestions, 10);
    }

    #[test]
    fn test_completion_item_builder() {
        let item = CompletionItem::new(
            "\\pos(100,200)".to_string(),
            "\\pos".to_string(),
            CompletionType::Tag,
        )
        .with_description("Position override tag".to_string())
        .with_detail("Sets absolute position".to_string())
        .with_sort_order(1);

        assert_eq!(item.insert_text, "\\pos(100,200)");
        assert_eq!(item.label, "\\pos");
        assert_eq!(item.description, Some("Position override tag".to_string()));
        assert_eq!(item.detail, Some("Sets absolute position".to_string()));
        assert_eq!(item.sort_order, 1);
    }

    #[test]
    fn test_max_suggestions_limit() {
        let mut ext = AutoCompleteExtension::new();
        ext.config.max_suggestions = 2;

        let doc = EditorDocument::from_content("[Script Info]\n").unwrap();
        let completions = ext
            .get_completions(&doc, Position::new(doc.len_bytes()))
            .unwrap();

        // Should respect max_suggestions
        assert_eq!(completions.len(), 2);
    }

    #[test]
    fn test_update_styles_command() {
        let mut ext = AutoCompleteExtension::new();
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::from_content(
            "[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: MyStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1",
        )
        .unwrap();
        let mut context = manager
            .create_context("test".to_string(), Some(&mut doc))
            .unwrap();

        ext.initialize(&mut *context).unwrap();

        // Execute update styles command
        let result = ext
            .execute_command("autocomplete.update_styles", &HashMap::new(), &mut *context)
            .unwrap();
        assert!(result.success);
        assert!(result.message.unwrap().contains("1 style names"));
    }

    #[test]
    fn test_unknown_command() {
        let mut ext = AutoCompleteExtension::new();
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::new();
        let mut context = manager
            .create_context("test".to_string(), Some(&mut doc))
            .unwrap();

        let result = ext
            .execute_command("unknown.command", &HashMap::new(), &mut *context)
            .unwrap();
        assert!(!result.success);
        assert!(result.message.unwrap().contains("Unknown command"));
    }

    #[test]
    fn test_animation_tag_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "{\\t".to_string(),
            column: 3,
            section: Some("Events".to_string()),
            in_override_tag: true,
            current_tag: Some("t".to_string()),
        };

        let completions = ext.get_tag_completions(&context);

        // Should have animation tag
        let anim = completions.iter().find(|c| c.label == "\\t").unwrap();
        assert_eq!(anim.insert_text, "\\t(\\fs30)");
        assert_eq!(anim.completion_type, CompletionType::Tag);
    }

    #[test]
    fn test_style_section_completions() {
        let ext = AutoCompleteExtension::new();
        let context = CompletionContext {
            line: "".to_string(),
            column: 0,
            section: Some("V4+ Styles".to_string()),
            in_override_tag: false,
            current_tag: None,
        };

        let completions = ext.get_field_completions("V4+ Styles", &context);

        // Should have Format and Style
        assert_eq!(completions.len(), 2);
        assert!(completions.iter().any(|c| c.label == "Format:"));
        assert!(completions.iter().any(|c| c.label == "Style:"));
    }
}
