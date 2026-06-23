// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Integration tests for the `reqx` CLI.
//!
//! These lock the observable behaviour of the binary (exit codes, output,
//! assertions against a real HTTP server) so later refactors stay honest.
//! HTTP is mocked with `wiremock`; the CLI is driven with `assert_cmd`.

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Write a `.reqx` file into `dir` and return its path.
fn write_reqx(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
    let file = dir.join(name);
    fs::write(&file, body).expect("write .reqx");
    file
}

fn reqx() -> Command {
    Command::cargo_bin("reqx").expect("binary `reqx` builds")
}

#[test]
fn init_creates_collection_structure() {
    let dir = TempDir::new().unwrap();
    reqx()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(
        dir.path().join(".reqx/config.toml").exists(),
        ".reqx/config.toml should be created by `init`"
    );
}

#[tokio::test]
async fn run_passes_when_status_matches() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let dir = TempDir::new().unwrap();
    let file = write_reqx(
        dir.path(),
        "ok.reqx",
        &format!(
            r#"
[request]
method = "GET"
url = "{}/users"

[assert]
status = "200"
"#,
            server.uri()
        ),
    );

    reqx()
        .arg("run")
        .arg(&file)
        .current_dir(dir.path())
        .assert()
        .success(); // exit 0
}

#[tokio::test]
async fn run_fails_when_assertion_mismatches() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/boom"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let dir = TempDir::new().unwrap();
    let file = write_reqx(
        dir.path(),
        "fail.reqx",
        &format!(
            r#"
[request]
method = "GET"
url = "{}/boom"

[assert]
status = "200"
"#,
            server.uri()
        ),
    );

    reqx()
        .arg("run")
        .arg(&file)
        .current_dir(dir.path())
        .assert()
        .code(1); // assertion failure => exit 1
}

#[tokio::test]
async fn run_asserts_jsonpath_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "id": 42 })))
        .mount(&server)
        .await;

    let dir = TempDir::new().unwrap();
    let file = write_reqx(
        dir.path(),
        "json.reqx",
        &format!(
            r#"
[request]
method = "GET"
url = "{}/item"

[assert]
status = "200"
"body.id" = "42"
"#,
            server.uri()
        ),
    );

    reqx()
        .arg("run")
        .arg(&file)
        .current_dir(dir.path())
        .assert()
        .success();
}

#[tokio::test]
async fn run_json_output_reports_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ping"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let dir = TempDir::new().unwrap();
    let file = write_reqx(
        dir.path(),
        "ping.reqx",
        &format!(
            r#"
[request]
method = "GET"
url = "{}/ping"

[assert]
status = "200"
"#,
            server.uri()
        ),
    );

    let out = reqx()
        .arg("run")
        .arg(&file)
        .arg("--output")
        .arg("json")
        .current_dir(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("status"), "json output should mention status");
}

#[tokio::test]
async fn run_wildcard_asserts_all_elements() {
    let server = MockServer::start().await;
    // All items active => wildcard assertion passes.
    Mock::given(method("GET"))
        .and(path("/ok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({ "items": [ { "active": true }, { "active": true } ] }),
        ))
        .mount(&server)
        .await;
    // One item inactive => wildcard assertion fails.
    Mock::given(method("GET"))
        .and(path("/bad"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({ "items": [ { "active": true }, { "active": false } ] }),
        ))
        .mount(&server)
        .await;

    let dir = TempDir::new().unwrap();
    let pass = write_reqx(
        dir.path(),
        "all.reqx",
        &format!(
            "[request]\nmethod = \"GET\"\nurl = \"{}/ok\"\n\n[assert]\n\"body.items[*].active\" = \"true\"\n",
            server.uri()
        ),
    );
    reqx()
        .arg("run")
        .arg(&pass)
        .current_dir(dir.path())
        .assert()
        .success();

    let fail = write_reqx(
        dir.path(),
        "notall.reqx",
        &format!(
            "[request]\nmethod = \"GET\"\nurl = \"{}/bad\"\n\n[assert]\n\"body.items[*].active\" = \"true\"\n",
            server.uri()
        ),
    );
    reqx()
        .arg("run")
        .arg(&fail)
        .current_dir(dir.path())
        .assert()
        .code(1);
}

#[tokio::test]
async fn run_parallel_executes_a_directory() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/a"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/b"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let dir = TempDir::new().unwrap();
    write_reqx(
        dir.path(),
        "a.reqx",
        &format!(
            "[request]\nmethod = \"GET\"\nurl = \"{}/a\"\n\n[assert]\nstatus = \"200\"\n",
            server.uri()
        ),
    );
    write_reqx(
        dir.path(),
        "b.reqx",
        &format!(
            "[request]\nmethod = \"GET\"\nurl = \"{}/b\"\n\n[assert]\nstatus = \"200\"\n",
            server.uri()
        ),
    );

    reqx()
        .arg("run")
        .arg(dir.path())
        .arg("--parallel")
        .arg("4")
        .current_dir(dir.path())
        .assert()
        .success();
}

#[test]
fn validate_accepts_a_well_formed_file() {
    let dir = TempDir::new().unwrap();
    let file = write_reqx(
        dir.path(),
        "good.reqx",
        r#"
[request]
method = "GET"
url = "https://example.com/x"

[assert]
status = "200"
"#,
    );

    reqx()
        .arg("validate")
        .arg(&file)
        .current_dir(dir.path())
        .assert()
        .success();
}
