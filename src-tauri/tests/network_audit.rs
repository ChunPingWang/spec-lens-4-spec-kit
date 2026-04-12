//! T160 — network audit.
//!
//! SC-010 demands that the SpecLens backend performs **zero outbound
//! network calls in steady-state** after the initial Spec-Kit environment
//! check. The env check itself shells out to the local `spec-kit`
//! binary, not to any remote endpoint — the bundled `latest_known`
//! manifest in `version_checker.rs` is the only reference string. This
//! integration test asserts that no Rust source file in `src-tauri/src`
//! uses any forbidden network client API, so any future regression that
//! wires in `reqwest::get` / `ureq::get` / raw sockets fails the build
//! instead of quietly phoning home.
//!
//! The audit is intentionally static — it reads source files and looks
//! for the well-known call patterns. This keeps the check hermetic
//! (no runtime DNS, no firewall access) and fast enough for CI.
//!
//! Non-source locations that **are** allowed to contain the tokens:
//!   * `tests/network_audit.rs` (this file — it lists the patterns).
//!   * `services/version_checker.rs` string literals for the Spec-Kit
//!     release-notes URL and install command (documented, never
//!     fetched at runtime).
//!   * comments and doc-strings.

use std::fs;
use std::path::{Path, PathBuf};

/// Call patterns that would imply an outbound network request from the
/// backend in steady-state. We scan `src-tauri/src` for these strings;
/// any hit fails the audit.
const FORBIDDEN_PATTERNS: &[&str] = &[
    // reqwest
    "reqwest::Client",
    "reqwest::get",
    "reqwest::blocking",
    "reqwest::RequestBuilder",
    // alt HTTP clients
    "ureq::get",
    "ureq::post",
    "ureq::Agent",
    "isahc::get",
    "isahc::post",
    "hyper::Client",
    "hyper_util::client",
    // raw sockets
    "TcpStream::connect",
    "TcpListener::bind",
    "UdpSocket::bind",
    "UdpSocket::connect",
    // tauri http plugin
    "tauri_plugin_http",
];

fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn walk_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Strip line-comments and doc-strings so a documentation reference to
/// `reqwest::get` (for example) does not fail the audit. This is a
/// lightweight line-by-line pass — a rule hit inside a `//!` or `//`
/// comment is ignored, everything else is fair game.
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        // Inline `// ...` — keep the code half.
        if let Some(idx) = line.find("//") {
            // Bail if the `//` sits inside a string literal; the audit
            // only cares about obvious source lines so this is good
            // enough for the patterns we look for.
            if !line[..idx].contains('"') {
                out.push_str(&line[..idx]);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[test]
fn no_forbidden_network_apis_in_backend_source() {
    let mut files = Vec::new();
    walk_rs_files(&src_dir(), &mut files);
    assert!(
        !files.is_empty(),
        "walk did not find any .rs files under src-tauri/src"
    );

    let mut hits: Vec<String> = Vec::new();
    for file in &files {
        let contents = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(err) => panic!("read {}: {err}", file.display()),
        };
        let code = strip_comments(&contents);
        for pattern in FORBIDDEN_PATTERNS {
            if code.contains(pattern) {
                hits.push(format!("{}: {pattern}", file.display()));
            }
        }
    }

    assert!(
        hits.is_empty(),
        "SC-010 network audit failed — forbidden outbound-network APIs:\n  {}",
        hits.join("\n  ")
    );
}

#[test]
fn version_checker_url_is_documentation_only() {
    // Pin the fact that the release-notes URL in version_checker.rs is a
    // string literal used only for display, never fetched. If someone
    // later wires it through an HTTP client the previous test will fire.
    let vc = src_dir().join("services").join("version_checker.rs");
    let contents = fs::read_to_string(&vc).expect("read version_checker.rs");
    assert!(
        contents.contains("https://github.com/github/spec-kit/releases"),
        "release notes URL missing from version_checker.rs"
    );
    // And the bundled manifest constant is a plain string literal.
    assert!(contents.contains("BUNDLED_LATEST"));
}

#[test]
fn audit_itself_lists_core_patterns() {
    // Smoke test so accidental deletions of the pattern list fail loudly.
    assert!(FORBIDDEN_PATTERNS.contains(&"reqwest::Client"));
    assert!(FORBIDDEN_PATTERNS.contains(&"TcpStream::connect"));
    assert!(FORBIDDEN_PATTERNS.len() >= 10);
}
