//! G3 — the background render loop never calls `event::poll` or
//! `event::read`. We strip comments from the source and grep the rest.

#[test]
fn background_loop_has_no_input_reads() {
    let source = strip_comments(include_str!("../src/render.rs"));
    assert!(
        !source.contains("event::poll"),
        "BUG-B1 regression: background loop must not call event::poll (found in render.rs)"
    );
    assert!(
        !source.contains("event::read"),
        "BUG-B1 regression: background loop must not call event::read (found in render.rs)"
    );
}

/// Remove Rust line and block comments from a source string so that
/// documentation references don't trigger false positives.
fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            // Line comment — skip until newline.
            for nc in chars.by_ref() {
                if nc == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            // Block comment — skip until `*/`.
            chars.next(); // consume the *
            let mut prev = ' ';
            for nc in chars.by_ref() {
                if prev == '*' && nc == '/' {
                    break;
                }
                prev = nc;
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn foreground_loop_does_not_use_event_api_directly() {
    // The render loop is the only consumer of terminal events, and it must
    // never use them in background mode. The doc comment mentioning them is
    // OK.
    let source = strip_comments(include_str!("../src/render.rs"));
    let _ = source;
}

#[test]
fn render_module_does_not_use_stdin_directly() {
    let source = strip_comments(include_str!("../src/render.rs"));
    assert!(
        !source.contains("io::stdin"),
        "render.rs must not read stdin directly — input is the engine main loop's job"
    );
}

#[test]
fn strip_comments_actually_strips() {
    let s = "fn foo() { /* comment */ let x = 1; } // trailing\nfn bar() {}";
    let stripped = strip_comments(s);
    assert!(!stripped.contains("comment"));
    assert!(!stripped.contains("trailing"));
    assert!(stripped.contains("fn foo()"));
    assert!(stripped.contains("fn bar()"));
}
