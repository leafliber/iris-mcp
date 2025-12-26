use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn server_starts_and_handles_basic_requests() {
    // 路径由 Cargo 在测试时注入，指向待测可执行文件
    let exe = env!("CARGO_BIN_EXE_iris-mcp");

    let mut child = Command::new(exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn server");

    let mut stdin = child.stdin.take().expect("stdin not available");
    let stdout = child.stdout.take().expect("stdout not available");
    let _stderr = child.stderr.take();

    // 将 stdout 读取放在单独线程，避免阻塞
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    // 发送 initialize
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    writeln!(stdin, "{}", init).expect("write initialize failed");
    stdin.flush().unwrap();

    let init_resp = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("no initialize response")
        .expect("failed to read initialize line");

    let init_json: serde_json::Value = serde_json::from_str(&init_resp).expect("invalid initialize JSON");
    assert_eq!(init_json["id"], 1);
    assert!(init_json["result"].is_object(), "initialize should return result");

    // 发送 tools/list
    let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    writeln!(stdin, "{}", list_req).expect("write tools/list failed");
    stdin.flush().unwrap();

    let list_resp = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("no tools/list response")
        .expect("failed to read tools/list line");
    let list_json: serde_json::Value = serde_json::from_str(&list_resp).expect("invalid tools/list JSON");
    assert_eq!(list_json["id"], 2);
    let tools = list_json["result"]["tools"].as_array().expect("tools should be array");
    assert!(!tools.is_empty(), "tools list should not be empty");

    // 发送 tools/call 缺少参数，预期返回 -32602 错误（验证调用管道工作）
    let call_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mouse_move","arguments":{}}}"#;
    writeln!(stdin, "{}", call_req).expect("write tools/call failed");
    stdin.flush().unwrap();

    let call_resp = rx
        .recv_timeout(Duration::from_secs(3))
        .expect("no tools/call response")
        .expect("failed to read tools/call line");
    let call_json: serde_json::Value = serde_json::from_str(&call_resp).expect("invalid tools/call JSON");
    assert_eq!(call_json["id"], 3);
    let err_code = call_json["error"]["code"].as_i64().expect("call should return error code");
    assert_eq!(err_code, -32602, "tools/call missing params should return -32602");

    // 结束子进程
    let _ = child.kill();
    let _ = child.wait();
}
