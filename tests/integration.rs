use assert_cmd::Command;
use predicates::str::contains;

fn pretty() -> Command {
    Command::cargo_bin("pretty").unwrap()
}

#[test]
fn basic_json_line_output() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"hello world"}"#)
        .assert()
        .success()
        .stdout(contains("INFO"))
        .stdout(contains("hello world"));
}

#[test]
fn error_level_output() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"error","msg":"something broke"}"#)
        .assert()
        .success()
        .stdout(contains("ERROR"))
        .stdout(contains("something broke"));
}

#[test]
fn timestamp_shortened() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"ok","time":"2024-06-15T14:30:00Z"}"#)
        .assert()
        .success()
        .stdout(contains("14:30:00"));
}

#[test]
fn extra_fields_shown() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"req","port":8080}"#)
        .assert()
        .success()
        .stdout(contains("port=8080"));
}

#[test]
fn raw_non_json_line_passed_through() {
    pretty()
        .arg("--no-color")
        .write_stdin("plain text not json")
        .assert()
        .success()
        .stdout(contains("plain text not json"));
}

#[test]
fn multiline_stacktrace_indented() {
    let input =
        "{\"level\":\"error\",\"msg\":\"crash\"}\ngoroutine 1 [running]:\nmain.go:42\n";
    pretty()
        .arg("--no-color")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("ERROR"))
        .stdout(contains("  goroutine 1 [running]:"))
        .stdout(contains("  main.go:42"));
}

#[test]
fn highlight_errors_flag() {
    pretty()
        .arg("--no-color")
        .arg("-e")
        .write_stdin(r#"{"level":"warn","msg":"connection error"}"#)
        .assert()
        .success()
        .stdout(contains("connection error"));
}

#[test]
fn expand_flag_expands_nested_json() {
    let input = r#"{"level":"info","msg":"ok","meta":"{\"user\":\"alice\"}"}"#;
    pretty()
        .arg("--no-color")
        .arg("-s")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(contains("alice"));
}

#[test]
fn lvl_alias_recognized() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"lvl":"debug","msg":"tracing"}"#)
        .assert()
        .success()
        .stdout(contains("DEBUG"))
        .stdout(contains("tracing"));
}

#[test]
fn trace_id_shown() {
    pretty()
        .arg("--no-color")
        .write_stdin(r#"{"level":"info","msg":"req","trace_id":"abc-123"}"#)
        .assert()
        .success()
        .stdout(contains("trace=abc-123"));
}
