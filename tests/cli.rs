//! End-to-end tests that drive the compiled `scrollcast` binary against the
//! bundled `testfiles/test_project` fixture.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn run(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("scrollcast")
        .unwrap()
        .args(args)
        .assert()
}

#[test]
fn converts_repo_to_markdown() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.md");
    run(&[
        "testfiles/test_project",
        "-o",
        out.to_str().unwrap(),
        "-f",
        "markdown",
        "-y",
    ])
    .success();

    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("## File Contents"));
    assert!(content.contains("main.rs"));
}

#[test]
fn converts_repo_to_valid_pdf() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.pdf");
    run(&[
        "testfiles/test_project",
        "-o",
        out.to_str().unwrap(),
        "-f",
        "pdf",
        "-y",
    ])
    .success();

    let bytes = fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"%PDF"), "output is not a PDF");
    // The whole fixture must be present, not a 10-line truncation.
    assert!(bytes.len() > 1000, "PDF is suspiciously small");
}

#[test]
fn infers_format_from_output_extension() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.html");
    // Note: no --format flag; it must be inferred from `.html`.
    run(&["testfiles/test_project", "-o", out.to_str().unwrap(), "-y"]).success();

    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("<!DOCTYPE html>") || content.contains("<html"),
        "extension inference did not produce HTML"
    );
}

#[test]
fn runs_non_interactively_without_a_tty() {
    // Without -y and with piped (non-terminal) stdin, the tool must proceed
    // rather than erroring with "not a terminal".
    let dir = tempdir().unwrap();
    let out = dir.path().join("out.md");
    Command::cargo_bin("scrollcast")
        .unwrap()
        .args([
            "testfiles/test_project",
            "-o",
            out.to_str().unwrap(),
            "-f",
            "markdown",
        ])
        .write_stdin("")
        .assert()
        .success();
    assert!(out.exists());
}
