//! Purpose:
//! Integration regressions for `fopen()` stream-context selection, propagation,
//! and restoration across literal, dynamic, successful, and failed opens.
//!
//! Called from:
//! - `cargo test --test codegen_tests stream_context_propagation` through Rust's test harness.
//!
//! Key details:
//! - HTTP fixtures bind only an ephemeral loopback port and accept a fixed
//!   number of requests, so the tests never depend on external networking.
//! - Expected precedence is explicit non-null, then request default; explicit
//!   empty contexts mask defaults, while omitted and explicit null use them.

use super::*;
use std::io::{Read, Write};

/// Starts a deterministic HTTP server that returns each request method as its body.
fn spawn_http_method_server(
    requests: usize,
) -> (std::thread::JoinHandle<()>, u16) {
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("context test: bind port");
    let port = listener
        .local_addr()
        .expect("context test: local address")
        .port();
    let handle = std::thread::spawn(move || {
        for _ in 0..requests {
            let (mut socket, _) = listener.accept().expect("context test: accept");
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .expect("context test: read timeout");
            let mut request = Vec::new();
            let mut byte = [0u8; 1];
            while socket.read(&mut byte).unwrap_or(0) == 1 {
                request.push(byte[0]);
                if request.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            let method = request
                .split(|byte| *byte == b' ')
                .next()
                .unwrap_or(b"");
            let response = format!(
                "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n",
                method.len()
            );
            socket
                .write_all(response.as_bytes())
                .expect("context test: response headers");
            socket
                .write_all(method)
                .expect("context test: response body");
        }
    });
    (handle, port)
}

/// Verifies explicit null is accepted on a literal early-return wrapper path.
#[test]
fn test_fopen_literal_memory_accepts_explicit_null_context() {
    let out = compile_and_run(
        r#"<?php
$stream = fopen("php://memory", "r+", false, null);
echo is_resource($stream) ? "resource" : "bad";
fclose($stream);
"#,
    );
    assert_eq!(out, "resource");
}

/// Verifies default, explicit, empty, and null contexts follow PHP precedence.
#[test]
fn test_fopen_context_precedence_default_explicit_empty_and_null() {
    let (server, port) = spawn_http_method_server(4);
    let out = compile_and_run(
        &r#"<?php
stream_context_set_default(["http" => ["method" => "POST"]]);
$explicit = stream_context_create(["http" => ["method" => "PUT"]]);
$empty = stream_context_create();

$defaultStream = fopen("http://127.0.0.1:PHP_TEST_PORT/default", "r");
$explicitStream = fopen("http://127.0.0.1:PHP_TEST_PORT/explicit", "r", false, $explicit);
$emptyStream = fopen("http://127.0.0.1:PHP_TEST_PORT/empty", "r", false, $empty);
$nullStream = fopen("http://127.0.0.1:PHP_TEST_PORT/null", "r", false, null);

echo stream_get_contents($defaultStream), "|";
echo stream_get_contents($explicitStream), "|";
echo stream_get_contents($emptyStream), "|";
echo stream_get_contents($nullStream);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "POST|PUT|GET|POST");
}

/// Verifies `file_get_contents()` honours the context passed to the call.
///
/// The path readers took their options from whichever context was published last
/// instead of their own `$context` argument, so every request went out as a GET no
/// matter what the caller built.
#[test]
fn test_file_get_contents_context_precedence_default_explicit_empty_and_null() {
    let (server, port) = spawn_http_method_server(4);
    let out = compile_and_run(
        &r#"<?php
stream_context_set_default(["http" => ["method" => "POST"]]);
$explicit = stream_context_create(["http" => ["method" => "PUT"]]);
$empty = stream_context_create();

echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/default"), "|";
echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/explicit", false, $explicit), "|";
echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/empty", false, $empty), "|";
echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/null", false, null);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "POST|PUT|GET|POST");
}

/// Verifies `readfile()` reaches an http URL at all, and honours its `$context`.
///
/// `readfile()` only ever opened its argument as a filesystem path, so a URL produced
/// an empty body and `false`. It now falls back to the URL reader on open failure,
/// which leaves ordinary files on the streaming path.
#[test]
fn test_readfile_reads_http_urls_and_honours_its_context() {
    let (server, port) = spawn_http_method_server(2);
    let out = compile_and_run(
        &r#"<?php
$put = stream_context_create(["http" => ["method" => "PUT"]]);
readfile("http://127.0.0.1:PHP_TEST_PORT/plain");
echo "|";
readfile("http://127.0.0.1:PHP_TEST_PORT/ctx", false, $put);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "GET|PUT");
}

/// Verifies a context passed to one read does not leak into the next one.
#[test]
fn test_file_get_contents_context_does_not_leak_to_later_reads() {
    let (server, port) = spawn_http_method_server(3);
    let out = compile_and_run(
        &r#"<?php
$put = stream_context_create(["http" => ["method" => "PUT"]]);
echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/a", false, $put), "|";
echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/b"), "|";
echo file_get_contents("http://127.0.0.1:PHP_TEST_PORT/c", false, $put);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "PUT|GET|PUT");
}

/// Verifies user wrappers receive the exact selected context in `$this->context`.
#[test]
fn test_fopen_user_wrapper_context_precedence_default_explicit_empty_and_null() {
    let out = compile_and_run(
        r#"<?php
class ContextProbeWrapper {
    public mixed $context = null;

    public function stream_open($path, $mode, $options, &$openedPath): bool {
        $contextOptions = stream_context_get_options($this->context);
        echo $contextOptions["probe"]["value"] ?? "-";
        return true;
    }
}

stream_wrapper_register("contextprobe", "ContextProbeWrapper");
stream_context_set_default(["probe" => ["value" => "D"]]);
$explicit = stream_context_create(["probe" => ["value" => "E"]]);
$empty = stream_context_create();

$stream = fopen("contextprobe://default", "r");
if (is_resource($stream)) fclose($stream); else echo "!";
echo "|";
$stream = fopen("contextprobe://explicit", "r", false, $explicit);
if (is_resource($stream)) fclose($stream); else echo "!";
echo "|";
$stream = fopen("contextprobe://empty", "r", false, $empty);
if (is_resource($stream)) fclose($stream); else echo "!";
echo "|";
$stream = fopen("contextprobe://null", "r", false, null);
if (is_resource($stream)) fclose($stream); else echo "!";
"#,
    );
    assert_eq!(out, "D|E|-|D");
}

/// Verifies context selection cannot clobber a dynamic filename or mode value.
#[test]
fn test_fopen_dynamic_path_and_mode_preserve_explicit_context() {
    let (server, port) = spawn_http_method_server(1);
    let out = compile_and_run(
        &r#"<?php
$context = stream_context_create(["http" => ["method" => "PUT"]]);
$suffix = "/dynamic";
$url = "http://127.0.0.1:PHP_TEST_PORT" . $suffix;
$mode = "r";
$stream = fopen($url, $mode, false, $context);
echo stream_get_contents($stream);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "PUT");
}

/// Verifies notifier selection follows default, explicit, empty, and null precedence.
#[test]
fn test_fopen_context_notifier_precedence_default_explicit_empty_and_null() {
    let (server, port) = spawn_http_method_server(4);
    let out = compile_and_run(
        &r#"<?php
$default = stream_context_get_default();
stream_context_set_params($default, ["notification" => function ($code) {
    if ($code === 2) echo "D";
}]);
$explicit = stream_context_create([], ["notification" => function ($code) {
    if ($code === 2) echo "E";
}]);
$empty = stream_context_create();

echo "[";
$defaultStream = fopen("http://127.0.0.1:PHP_TEST_PORT/default", "r");
fclose($defaultStream);
echo "]|[";
$explicitStream = fopen("http://127.0.0.1:PHP_TEST_PORT/explicit", "r", false, $explicit);
fclose($explicitStream);
echo "]|[";
$emptyStream = fopen("http://127.0.0.1:PHP_TEST_PORT/empty", "r", false, $empty);
fclose($emptyStream);
echo "]|[";
$nullStream = fopen("http://127.0.0.1:PHP_TEST_PORT/null", "r", false, null);
fclose($nullStream);
echo "]";
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "[D]|[E]|[]|[D]");
}

/// Verifies a failed explicit open restores the request-default option bridge.
#[test]
fn test_fopen_failed_explicit_context_restores_default_scope() {
    let (server, port) = spawn_http_method_server(1);
    let out = compile_and_run(
        &r#"<?php
stream_context_set_default(["http" => ["method" => "POST"]]);
$explicit = stream_context_create(["http" => ["method" => "PUT"]]);
$failed = @fopen("http://127.0.0.1:1/fail", "r", false, $explicit);
$stream = fopen("http://127.0.0.1:PHP_TEST_PORT/after", "r");
echo $failed === false ? "false|" : "bad|";
echo stream_get_contents($stream);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    server.join().expect("context test: server join");
    assert_eq!(out, "false|POST");
}
