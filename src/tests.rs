use super::*;
use serde_json::{Value, json};

fn items() -> Vec<ProcItem> {
    vec![
        ProcItem {
            pid: 10,
            name: "worker".into(),
            cpu: 3661,
            rss: 1536,
            uptime: 172800,
            warn: false,
            msg: String::new(),
        },
        ProcItem {
            pid: 20,
            name: "other".into(),
            cpu: 12,
            rss: 4096,
            uptime: 60,
            warn: true,
            msg: String::new(),
        },
        ProcItem {
            pid: 0,
            name: "missing".into(),
            cpu: 0,
            rss: 0,
            uptime: 0,
            warn: false,
            msg: "service not running".into(),
        },
    ]
}

fn report(json: bool, items: Vec<ProcItem>, sort: Option<SortBy>, limit: Option<usize>) -> String {
    colored::control::set_override(false);
    let mut output = Vec::new();
    {
        let format = if json { fmt::Format::json() } else { fmt::Format::text() };
        let mut format = format.with_output(&mut output);
        format.json_open("", false, true);
        printitems(&mut format, items, sort, limit);
        format.json_close(false);
        format.finish();
    }
    String::from_utf8(output).unwrap()
}

#[test]
fn json_report_fixture() {
    let output = report(true, items(), None, None);
    assert_eq!(output, include_str!("../tests/fixtures/report.json"));
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["services"].as_array().unwrap().len(), 3);
}

#[test]
fn empty_and_limited_reports_are_valid_json() {
    for (items, limit) in [(vec![], None), (items(), Some(0))] {
        let output = report(true, items, None, limit);
        assert_eq!(serde_json::from_str::<Value>(&output).unwrap(), json!({"services": []}));
    }
}

#[test]
fn services_sort_before_limit() {
    let unsorted = || {
        [(10, 10, 20, 20), (20, 30, 10, 30), (30, 20, 30, 10)]
            .into_iter()
            .map(|(pid, cpu, rss, uptime)| ProcItem {
                pid,
                name: format!("service-{pid}"),
                cpu,
                rss,
                uptime,
                warn: false,
                msg: String::new(),
            })
            .collect()
    };
    for (sort, expected) in [
        (SortBy::Cpu, [20, 30, 10]),
        (SortBy::Rss, [30, 10, 20]),
        (SortBy::Uptime, [20, 10, 30]),
    ] {
        for limit in [None, Some(1), Some(2)] {
            let output = report(true, unsorted(), Some(sort), limit);
            let value: Value = serde_json::from_str(&output).unwrap();
            let pids: Vec<u64> = value["services"]
                .as_array()
                .unwrap()
                .iter()
                .map(|service| service["pid"].as_u64().unwrap())
                .collect();
            assert_eq!(pids, expected[..limit.unwrap_or(expected.len())], "{sort:?}, limit {limit:?}");
        }
    }
}

#[test]
fn explicit_service_warning_is_preserved() {
    let mut items = items();
    items[0].warn = true;
    items[0].msg = "invalid PID file /run/worker.pid".into();
    let output = report(true, items, None, Some(1));
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["services"][0]["warning"], "invalid PID file /run/worker.pid");
}

#[test]
fn text_rows_fixture() {
    colored::control::set_override(false);
    let mut output = Vec::new();
    {
        let mut format = fmt::Format::text().with_output(&mut output);
        format.text_proc_ok("worker [10]".into(), "cpu 01:01:01 ∙ up 2 days ∙ 1.5 KiB rss".into());
        format.text_proc_warn("other [20]".into(), "cpu 00:00:12 ∙ up 00:01:00 ∙ 4 KiB rss".into());
        format.text_proc_err("missing [0]".into(), "service not running".into());
    }
    assert_eq!(String::from_utf8(output).unwrap(), include_str!("../tests/fixtures/report.txt"));
}

#[test]
fn text_report_has_no_json_delimiters() {
    assert_eq!(report(false, items(), None, None), include_str!("../tests/fixtures/report.txt"));
}

#[test]
fn json_report_escapes_service_strings() {
    let name = "worker \"quoted\" \\ path\n\t\r\u{0000} café";
    let warning = "PID file \"/run/a\\b\"\nmissing";
    let error = "error\t\u{001f}";
    let mut items = items();
    items[0].name = name.into();
    items[1].msg = warning.into();
    items[2].msg = error.into();
    let output = report(true, items, None, None);
    let value: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["services"][0]["name"], name);
    assert_eq!(value["services"][1]["warning"], warning);
    assert_eq!(value["services"][2]["error"], error);
}

#[test]
fn elapsed_formats_boundaries() {
    for (seconds, expected) in [
        (0, "00:00:00"),
        (59, "00:00:59"),
        (3600, "01:00:00"),
        (86399, "23:59:59"),
        (86400, "1 days"),
    ] {
        assert_eq!(elapsed(seconds), expected);
    }
}

#[test]
fn failed_collection_does_not_publish_partial_json() {
    let mut output = Vec::new();
    let mut format = fmt::Format::json().with_output(&mut output);
    let result = write_report(&mut format, |format| {
        format.json_open("", false, true);
        format.json_key_val("cores", 4);
        Err::<(), _>("collection failed")
    });
    assert_eq!(result, Err("collection failed"));
    drop(format);
    assert!(output.is_empty());
}
