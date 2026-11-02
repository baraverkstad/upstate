use serde_json::{Value, json};
use std::path::Path;
use std::process::{Command, Output};

fn run(config: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_upstate"))
        .args(args)
        .env(
            "UPSTATE_CONF",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config").join(config),
        )
        .env("NO_COLOR", "1")
        .env("CLICOLOR_FORCE", "0")
        .output()
        .unwrap()
}

fn json_output(output: &Output, code: i32) -> Value {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let mut value: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| panic!("{err}: {output:?}"));
    let collected = value
        .as_object_mut()
        .unwrap()
        .remove("collected")
        .expect("missing collected timestamp");
    let timestamp = collected.as_str().unwrap();
    assert_eq!(timestamp.len(), 20);
    assert!(timestamp.ends_with('Z'));
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .ends_with(&format!("\"collected\":\"{timestamp}\"}}"))
    );
    value
}

#[test]
fn help_and_version_exit_successfully_without_a_report() {
    for flag in ["--help", "-h", "-?", "--version"] {
        let output = run("empty.toml", &[flag]);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let message = String::from_utf8(output.stderr).unwrap();
        assert!(message.contains(if flag == "--version" { "Upstate (" } else { "Syntax: upstate" }));
    }
}

#[test]
fn invalid_arguments_exit_one_and_explain_the_error() {
    for (flag, message) in [
        ("--unknown", "invalid command-line argument"),
        ("--sort=invalid", "invalid sort option"),
        ("--sort=", "invalid sort option"),
        ("--limit=invalid", "invalid limit option"),
        ("--limit=-1", "invalid limit option"),
        ("--limit=", "invalid limit option"),
    ] {
        let output = run("empty.toml", &[flag]);
        assert_eq!(output.status.code(), Some(1), "{flag}: {output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(message), "{output:?}");
    }
}

#[test]
fn json_can_suppress_both_sections() {
    let output = run("missing.toml", &["--json", "--no-summary", "--no-services"]);
    assert_eq!(json_output(&output, 0), json!({}));
}

#[test]
fn sort_aliases_and_limits_are_accepted() {
    for sort in ["cpu", "rss", "mem", "uptime", "time"] {
        let output = run(
            "empty.toml",
            &["--json", "--no-summary", "--limited", &format!("--sort={sort}"), "--limit=0"],
        );
        assert_eq!(json_output(&output, 0), json!({"services": []}));
    }
}

#[test]
fn missing_required_services_set_exit_code_even_when_hidden_by_limit() {
    let output = run("missing.toml", &["--json", "--no-summary", "--limited"]);
    assert_eq!(
        json_output(&output, 2),
        json!({"services": [
            {"pid": 0, "name": "missing-one", "error": "service not running"},
            {"pid": 0, "name": "missing-two", "error": "service not running"}
        ]})
    );
    let output = run("missing.toml", &["--json", "--no-summary", "--limited", "--limit=0"]);
    assert_eq!(json_output(&output, 2), json!({"services": []}));
}

#[test]
fn absent_optional_services_do_not_fail() {
    let output = run("optional.toml", &["--json", "--no-summary", "--limited"]);
    assert_eq!(json_output(&output, 0), json!({"services": []}));
}

#[test]
fn last_service_display_flag_wins() {
    let output = run("missing.toml", &["--json", "--no-summary", "--no-services", "--limited"]);
    assert_eq!(json_output(&output, 2)["services"].as_array().unwrap().len(), 2);
    let output = run("missing.toml", &["--json", "--no-summary", "--limited", "--no-services"]);
    assert_eq!(json_output(&output, 0), json!({}));
}

#[cfg(target_os = "linux")]
#[test]
fn linux_smoke_collects_summary_and_discovers_services() {
    let output = run("empty.toml", &["--json", "--no-services"]);
    let raw = std::str::from_utf8(&output.stdout).unwrap();
    let fields = [
        "hostname",
        "os",
        "kernel",
        "architecture",
        "cores",
        "cpus",
        "uptime",
        "loadavg",
        "processes",
        "memory",
        "storage",
        "collected",
    ];
    let positions: Vec<_> = fields.iter().map(|field| raw.find(&format!("\"{field}\":")).unwrap()).collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    let summary = json_output(&output, 0);
    assert!(summary["cores"].as_u64().unwrap() >= 1);
    assert!(summary["uptime"].is_u64());
    assert_eq!(summary["loadavg"].as_array().unwrap().len(), 3);
    assert!(summary["processes"].as_u64().unwrap() > 0);
    let memory = &summary["memory"];
    for key in ["total", "avail", "used", "swap"] {
        assert!(memory[key].is_u64(), "{key}: {memory}");
    }
    assert!(memory["total"].as_u64().unwrap() > 0);
    assert!(memory["avail"].as_u64().unwrap() <= memory["total"].as_u64().unwrap());
    for disk in summary["storage"].as_array().unwrap() {
        assert_eq!(
            disk["total"].as_u64().unwrap(),
            disk["used"].as_u64().unwrap() + disk["free"].as_u64().unwrap()
        );
        assert!(disk["dev"].is_string());
        assert!(disk["mount"].is_string());
    }
    assert!(summary.get("services").is_none());

    let output = run("empty.toml", &["--json", "--no-summary", "--limited", "--complete"]);
    let complete = json_output(&output, 0);
    assert!(complete.get("memory").is_none());
    assert!(!complete["services"].as_array().unwrap().is_empty());

    let output = run("self.toml", &["--json", "--no-summary", "--limited"]);
    let matched = json_output(&output, 0);
    assert_eq!(matched["services"].as_array().unwrap().len(), 1);
    let service = &matched["services"][0];
    assert_eq!(service["name"], "upstate-self");
    assert!(service["pid"].as_u64().unwrap() > 0);
    assert!(service["cputime"].is_u64());
    assert!(service["uptime"].is_u64());
    assert!(service["rss"].as_u64().unwrap() > 0);
    assert!(service.get("warning").is_none());
}
