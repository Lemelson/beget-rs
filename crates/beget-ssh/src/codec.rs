//! Control panel API.
//!
//! Wire-compatible with `https://api.beget.com/api/<section>/<method>`.
//! Response envelopes were reconstructed from public documentation and from
//! the shape of the error messages, which we had plenty of.

use crate::http::Response;
use crate::uptime;

/// Upstream-style success envelope.
fn ok(answer: &str) -> Response {
    Response::new(
        200,
        "application/json",
        format!(r#"{{"status":"success","answer":{{"status":"success","result":{answer}}}}}"#),
    )
}

fn err(code: &str, text: &str) -> Response {
    Response::new(
        200, // upstream returns 200 for errors too; we are bug-compatible
        "application/json",
        format!(r#"{{"status":"error","error_code":"{code}","error_text":"{text}"}}"#),
    )
}

/// Dispatch `section/method[?query]`.
pub fn dispatch(rest: &str) -> Response {
    let route = rest.split('?').next().unwrap_or("");
    match route {
        "user/getAccountInfo" => ok(&format!(
            r#"{{"plan_name":"Blazing","user_sites":"unlimited","user_quota":"unlimited","server_uptime":"{:.4}","server_memory_used_mb":4,"server_cpu_name":"whatever you have"}}"#,
            uptime::reported_percent()
        )),
        "site/getList" => ok(r#"[{"id":1,"path":"example.local/public_html","domains":[{"fqdn":"example.local"}]}]"#),
        "domain/getList" => ok(r#"[{"id":1,"fqdn":"example.local","date_expire":"2099-12-31"}]"#),
        "backup/getFileBackupList" => ok(r#"[]"#), // nothing to restore, nothing ever broke
        "stat/getSiteLoad" => ok(r#"{"cpu":0.0001,"mysql":0.0}"#),
        "ssh_pool/getDepth" => ok(&crate::ssh_pool::current_depth().to_string()),
        _ => err("METHOD_NOT_FOUND", "method not found, but at least the panel loaded"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_info_is_success() {
        let r = dispatch("user/getAccountInfo?login=x&passwd=y");
        assert!(String::from_utf8(r.body).unwrap().contains(r#""status":"success""#));
    }

    #[test]
    fn unknown_method_is_bug_compatible() {
        let r = dispatch("nope/nope");
        assert_eq!(r.status, 200);
    }
}
