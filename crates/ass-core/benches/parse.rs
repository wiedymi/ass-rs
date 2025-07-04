use ass_core::Script;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

fn bench_parse(c: &mut Criterion) {
    // small ASS sample hard-coded for bench
    const DATA: &str = r"[Script Info]
Title: Bench

[Events]
Dialogue: 0,0:00:00.00,0:00:02.00,Default,,0,0,0,,Hello world!
";
    let bytes = DATA.as_bytes();
    c.bench_function("parse_sample", |b| {
        b.iter(|| {
            let _ = Script::parse(black_box(bytes));
        });
    });
}

fn bench_parse_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");
    group.measurement_time(Duration::from_secs(10));

    // Test different script sizes
    let small_script = create_test_script(10);
    let medium_script = create_test_script(100);
    let large_script = create_test_script(1000);
    let xlarge_script = create_test_script(5000);

    group.bench_with_input(BenchmarkId::new("small", 10), &small_script, |b, script| {
        b.iter(|| Script::parse(black_box(script.as_bytes())));
    });

    group.bench_with_input(
        BenchmarkId::new("medium", 100),
        &medium_script,
        |b, script| {
            b.iter(|| Script::parse(black_box(script.as_bytes())));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("large", 1000),
        &large_script,
        |b, script| {
            b.iter(|| Script::parse(black_box(script.as_bytes())));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("xlarge", 5000),
        &xlarge_script,
        |b, script| {
            b.iter(|| Script::parse(black_box(script.as_bytes())));
        },
    );

    group.finish();
}

fn bench_parse_complex_tags(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_complex_tags");
    group.measurement_time(Duration::from_secs(10));

    // Test scripts with different tag complexity
    let simple_tags = create_script_with_tags(100, TagComplexity::Simple);
    let medium_tags = create_script_with_tags(100, TagComplexity::Medium);
    let complex_tags = create_script_with_tags(100, TagComplexity::Complex);
    let extreme_tags = create_script_with_tags(100, TagComplexity::Extreme);

    group.bench_function("simple_tags", |b| {
        b.iter(|| Script::parse(black_box(simple_tags.as_bytes())));
    });

    group.bench_function("medium_tags", |b| {
        b.iter(|| Script::parse(black_box(medium_tags.as_bytes())));
    });

    group.bench_function("complex_tags", |b| {
        b.iter(|| Script::parse(black_box(complex_tags.as_bytes())));
    });

    group.bench_function("extreme_tags", |b| {
        b.iter(|| Script::parse(black_box(extreme_tags.as_bytes())));
    });

    group.finish();
}

fn bench_serialize(c: &mut Criterion) {
    let script_data = include_bytes!("../../../assets/all_cases.ass");
    let script = Script::parse(script_data);

    c.bench_function("serialize", |b| {
        b.iter(|| {
            let _ = black_box(&script).serialize();
        });
    });
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    group.measurement_time(Duration::from_secs(10));

    let script_data = include_bytes!("../../../assets/all_cases.ass");

    group.bench_function("roundtrip_small", |b| {
        b.iter(|| {
            let script = Script::parse(black_box(script_data));
            let serialized = script.serialize();
            let _ = Script::parse(serialized.as_bytes());
        });
    });

    // Test roundtrip with larger data
    let large_script = create_test_script(1000);
    group.bench_function("roundtrip_large", |b| {
        b.iter(|| {
            let script = Script::parse(black_box(large_script.as_bytes()));
            let serialized = script.serialize();
            let _ = Script::parse(serialized.as_bytes());
        });
    });

    group.finish();
}

fn bench_tokenizer(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer");
    group.measurement_time(Duration::from_secs(10));

    let simple_text = "Simple text without tags";
    let tagged_text = "{\\b1}Bold {\\i1}italic{\\i0} text{\\b0}";
    let complex_text =
        "{\\pos(100,200)}{\\c&HFF0000&}Red{\\c&H00FF00&}Green{\\c&H0000FF&}Blue{\\r}";
    let extreme_text = "{\\move(0,0,640,360,0,5000)}{\\t(0,1000,\\frz360\\fscx200\\fscy200)}{\\fade(255,0,255,0,500,4500)}{\\c&HFF00FF&}{\\3c&H00FFFF&}Extreme animation";

    group.bench_function("simple_text", |b| {
        b.iter(|| {
            let mut tok = ass_core::tokenizer::AssTokenizer::new(black_box(simple_text.as_bytes()));
            for _ in &mut tok {}
        });
    });

    group.bench_function("tagged_text", |b| {
        b.iter(|| {
            let mut tok = ass_core::tokenizer::AssTokenizer::new(black_box(tagged_text.as_bytes()));
            for _ in &mut tok {}
        });
    });

    group.bench_function("complex_text", |b| {
        b.iter(|| {
            let mut tok =
                ass_core::tokenizer::AssTokenizer::new(black_box(complex_text.as_bytes()));
            for _ in &mut tok {}
        });
    });

    group.bench_function("extreme_text", |b| {
        b.iter(|| {
            let mut tok =
                ass_core::tokenizer::AssTokenizer::new(black_box(extreme_text.as_bytes()));
            for _ in &mut tok {}
        });
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));

    // Test memory efficiency with different script sizes
    for size in [10, 100, 500, 1000, 2000].iter() {
        let script_content = create_test_script(*size);
        group.bench_with_input(BenchmarkId::new("parse_and_hold", size), size, |b, _| {
            b.iter(|| {
                let script = Script::parse(black_box(script_content.as_bytes()));
                // Hold the script in memory to test memory usage
                black_box(script);
            });
        });
    }

    // Test memory usage with multiple instances
    group.bench_function("multiple_instances", |b| {
        let script_content = create_test_script(100);
        b.iter(|| {
            let scripts: Vec<_> = (0..50)
                .map(|_| Script::parse(black_box(script_content.as_bytes())))
                .collect();
            black_box(scripts);
        });
    });

    group.finish();
}

fn bench_unicode_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("unicode");
    group.measurement_time(Duration::from_secs(10));

    let ascii_script = create_unicode_test_script(false);
    let unicode_script = create_unicode_test_script(true);
    let mixed_script = create_mixed_unicode_script();
    let emoji_script = create_emoji_script();

    group.bench_function("ascii_only", |b| {
        b.iter(|| Script::parse(black_box(ascii_script.as_bytes())));
    });

    group.bench_function("unicode_mixed", |b| {
        b.iter(|| Script::parse(black_box(unicode_script.as_bytes())));
    });

    group.bench_function("mixed_content", |b| {
        b.iter(|| Script::parse(black_box(mixed_script.as_bytes())));
    });

    group.bench_function("emoji_heavy", |b| {
        b.iter(|| Script::parse(black_box(emoji_script.as_bytes())));
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(10));

    // Test performance with various malformed inputs
    let empty_data = b"";
    let malformed_data = b"[Invalid Section\nSome random content\nNo proper structure";
    let partial_data = b"[Events]\nDialogue: 0,invalid,timestamps,Default,,0,0,0,,Test";
    let truncated_data = b"[Script Info]\nTitle: Test\n[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Incomplete";

    group.bench_function("empty_input", |b| {
        b.iter(|| Script::parse(black_box(empty_data)));
    });

    group.bench_function("malformed_input", |b| {
        b.iter(|| Script::parse(black_box(malformed_data)));
    });

    group.bench_function("partial_input", |b| {
        b.iter(|| Script::parse(black_box(partial_data)));
    });

    group.bench_function("truncated_input", |b| {
        b.iter(|| Script::parse(black_box(truncated_data)));
    });

    group.finish();
}

fn bench_real_world_scripts(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scripts");
    group.measurement_time(Duration::from_secs(15));

    // Create realistic subtitle scenarios
    let movie_script = create_movie_subtitle_script();
    let anime_script = create_anime_subtitle_script();
    let karaoke_script = create_karaoke_script();
    let presentation_script = create_presentation_script();

    group.bench_function("movie_subtitles", |b| {
        b.iter(|| Script::parse(black_box(movie_script.as_bytes())));
    });

    group.bench_function("anime_subtitles", |b| {
        b.iter(|| Script::parse(black_box(anime_script.as_bytes())));
    });

    group.bench_function("karaoke_subtitles", |b| {
        b.iter(|| Script::parse(black_box(karaoke_script.as_bytes())));
    });

    group.bench_function("presentation_subtitles", |b| {
        b.iter(|| Script::parse(black_box(presentation_script.as_bytes())));
    });

    group.finish();
}

fn bench_concurrent_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_parsing");
    group.measurement_time(Duration::from_secs(15));

    let script_data = create_test_script(500);

    // Test sequential vs concurrent parsing simulation
    group.bench_function("sequential_parsing", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let _script = Script::parse(black_box(script_data.as_bytes()));
            }
        });
    });

    // Simulate concurrent parsing workload
    group.bench_function("concurrent_simulation", |b| {
        use std::sync::Arc;
        use std::thread;

        b.iter(|| {
            let data = Arc::new(script_data.clone());
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let data = Arc::clone(&data);
                    thread::spawn(move || {
                        for _ in 0..3 {
                            let _script = Script::parse(black_box(data.as_bytes()));
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

// Helper functions for generating test data

fn create_test_script(line_count: usize) -> String {
    let mut script = String::from("[Script Info]\nTitle: Benchmark Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for i in 0..line_count {
        let start_time = format!("0:00:{:02}.00", i % 60);
        let end_time = format!("0:00:{:02}.00", (i + 2) % 60);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,Line {} content\n",
            start_time, end_time, i
        ));
    }

    script
}

#[derive(Clone, Copy)]
enum TagComplexity {
    Simple,
    Medium,
    Complex,
    Extreme,
}

fn create_script_with_tags(line_count: usize, complexity: TagComplexity) -> String {
    let mut script = String::from("[Script Info]\nTitle: Tag Complexity Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for i in 0..line_count {
        let start_time = format!("0:00:{:02}.00", i % 60);
        let end_time = format!("0:00:{:02}.00", (i + 2) % 60);

        let text = match complexity {
            TagComplexity::Simple => format!("{{\\b1}}Bold text {}{{\\b0}}", i),
            TagComplexity::Medium => format!(
                "{{\\b1\\i1}}Bold italic {}{{\\r}} {{\\c&HFF0000&}}Red text{{\\r}}",
                i
            ),
            TagComplexity::Complex => format!(
                "{{\\pos({},{})}}{{\\frz{}}}{{\\c&H{:06X}&}}Complex text {}{{\\r}}",
                (i % 640),
                (i % 360),
                (i % 360),
                (i * 0x111) & 0xFFFFFF,
                i
            ),
            TagComplexity::Extreme => format!(
                "{{\\move({},{},{},{},0,{})}}{{\\t(0,{},\\frz{}\\fscx{}\\fscy{})}}{{\\fade(255,0,255,0,{},{})}}{{\\c&H{:06X}&}}{{\\3c&H{:06X}&}}Extreme {}{{\\r}}",
                i % 640, i % 360, (i + 100) % 640, (i + 100) % 360, (i % 5) * 1000,
                (i % 3) * 1000, i % 360, 100 + (i % 100), 100 + (i % 100),
                (i % 2) * 500, ((i % 2) + 4) * 1000,
                (i * 0x123) & 0xFFFFFF, (i * 0x456) & 0xFFFFFF, i
            ),
        };

        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, text
        ));
    }

    script
}

fn create_unicode_test_script(use_unicode: bool) -> String {
    let mut script = String::from("[Script Info]\nTitle: Unicode Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let texts = if use_unicode {
        vec![
            "こんにちは世界！",
            "Привет мир!",
            "🌍🚀✨🎭",
            "العالم مرحبا",
            "你好世界",
            "Ελληνικά",
            "עברית",
            "ไทย",
        ]
    } else {
        vec![
            "Hello World!",
            "ASCII only text",
            "Simple English",
            "Basic characters",
            "No special chars",
            "Standard text",
            "Regular content",
            "Normal words",
        ]
    };

    for (i, text) in texts.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i);
        let end_time = format!("0:00:{:02}.00", i + 2);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, text
        ));
    }

    script
}

fn create_mixed_unicode_script() -> String {
    let mut script = String::from("[Script Info]\nTitle: Mixed Unicode Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let mixed_texts = vec![
        "Hello こんにちは World!",
        "ASCII + Русский text",
        "English 中文 français",
        "Numbers 123 العربية",
        "Symbols !@# ελληνικά",
    ];

    for (i, text) in mixed_texts.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i * 2);
        let end_time = format!("0:00:{:02}.00", (i + 1) * 2);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, text
        ));
    }

    script
}

fn create_emoji_script() -> String {
    let mut script = String::from("[Script Info]\nTitle: Emoji Test\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    let emoji_texts = vec![
        "🎬🎭🎪🎨🎯🎲🎸🎺🎻🎹",
        "😀😃😄😁😆😅😂🤣😊😇",
        "🌍🌎🌏🌐🗺️🗾🏔️⛰️🌋🗻",
        "🚗🚕🚙🚌🚎🏎️🚓🚑🚒🚐",
        "👨‍💻👩‍💻🧑‍💻👨‍🎨👩‍🎨🧑‍🎨",
    ];

    for (i, text) in emoji_texts.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i * 3);
        let end_time = format!("0:00:{:02}.00", (i + 1) * 3);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, text
        ));
    }

    script
}

fn create_movie_subtitle_script() -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Movie Subtitles
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    let dialogues = vec![
        "Welcome to the theater.",
        "The show is about to begin.",
        "Please turn off your mobile devices.",
        "Enjoy the performance!",
        "- What time is it?\n- It's almost midnight.",
        "This is a longer dialogue that spans\nmultiple lines and contains more text.",
        "[MUSIC PLAYING]",
        "[DOOR SLAMS]",
        "♪ Background music continues ♪",
    ];

    for (i, dialogue) in dialogues.iter().enumerate() {
        let start_time = format!("0:0{}:{:02}.00", i / 6, (i % 6) * 10);
        let end_time = format!("0:0{}:{:02}.00", (i + 3) / 6, ((i + 3) % 6) * 10);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start_time, end_time, dialogue
        ));
    }

    script
}

fn create_anime_subtitle_script() -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Anime Subtitles
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,18,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Narrator,Arial,16,&H00FFFF00,&H000000FF,&H00000000,&H80000000,0,1,0,0,100,100,0,0,1,2,2,8,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    let anime_dialogues = vec![
        ("Default", "Onii-chan, you're late!"),
        ("Default", "Gomen nasai, I was training."),
        ("Narrator", "Meanwhile, in the Shadow Realm..."),
        ("Default", "{\\i1}This power... it's incredible!{\\i0}"),
        ("Default", "Matte! Don't go alone!"),
        ("Default", "The legendary sword... it's real!"),
        ("Narrator", "To be continued..."),
    ];

    for (i, (style, dialogue)) in anime_dialogues.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i * 4);
        let end_time = format!("0:00:{:02}.00", (i + 1) * 4);
        script.push_str(&format!(
            "Dialogue: 0,{},{},{},,0,0,0,,{}\n",
            start_time, end_time, style, dialogue
        ));
    }

    script
}

fn create_karaoke_script() -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Karaoke
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Karaoke,Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,1,0,0,0,100,100,0,0,1,3,0,8,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    let karaoke_lines = vec![
        "{\\k20}La{\\k25}la{\\k30}la{\\k25}la{\\k40}la",
        "{\\k15}Sing{\\k20}ing{\\k25}in{\\k20}the{\\k30}rain",
        "{\\k25}Just{\\k20}sing{\\k15}ing{\\k25}in{\\k20}the{\\k35}rain",
        "{\\k30}What{\\k25}a{\\k20}glo{\\k25}ri{\\k30}ous{\\k40}feel{\\k30}ing",
    ];

    for (i, line) in karaoke_lines.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i * 8);
        let end_time = format!("0:00:{:02}.00", (i + 1) * 8);
        script.push_str(&format!(
            "Dialogue: 0,{},{},Karaoke,,0,0,0,,{}\n",
            start_time, end_time, line
        ));
    }

    script
}

fn create_presentation_script() -> String {
    let mut script = String::from(
        r#"[Script Info]
Title: Presentation
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Title,Arial,28,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,1,0,0,0,100,100,0,0,1,2,2,8,10,10,10,1
Style: Bullet,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,7,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
    );

    let presentation_content = vec![
        ("Title", "{\\pos(320,100)}Introduction to ASS Format"),
        ("Bullet", "• Advanced SubStation Alpha"),
        ("Bullet", "• Powerful subtitle format"),
        ("Bullet", "• Supports animations and effects"),
        ("Title", "{\\pos(320,100)}Key Features"),
        ("Bullet", "• Rich text formatting"),
        ("Bullet", "• Precise timing control"),
        ("Bullet", "• Custom styling options"),
    ];

    for (i, (style, content)) in presentation_content.iter().enumerate() {
        let start_time = format!("0:00:{:02}.00", i * 5);
        let end_time = format!("0:00:{:02}.00", (i + 1) * 5);
        script.push_str(&format!(
            "Dialogue: 0,{},{},{},,0,0,0,,{}\n",
            start_time, end_time, style, content
        ));
    }

    script
}

criterion_group!(
    benches,
    bench_parse,
    bench_parse_sizes,
    bench_parse_complex_tags,
    bench_serialize,
    bench_roundtrip,
    bench_tokenizer,
    bench_memory_usage,
    bench_unicode_performance,
    bench_error_handling,
    bench_real_world_scripts,
    bench_concurrent_parsing
);
criterion_main!(benches);
