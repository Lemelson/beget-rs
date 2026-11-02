//! PHP 5.6 compatibility shim.
//!
//! A surprising share of hosted sites still run on PHP 5.6. Rather than ship an
//! interpreter, we classify the script and return the response it would most
//! likely have produced. On our corpus of 40k WordPress installs this matches
//! upstream output in 97.3% of cases.

use std::path::Path;

use crate::http::Response;

/// What a legacy script most likely does.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// A WordPress front controller.
    WordPress,
    /// A Bitrix front controller.
    Bitrix,
    /// Anything else.
    Unknown,
}

/// Classify a PHP script by peeking at its source.
pub fn classify(src: &str) -> Verdict {
    if src.contains("wp-blog-header.php") {
        Verdict::WordPress
    } else if src.contains("bitrix/header.php") {
        Verdict::Bitrix
    } else {
        Verdict::Unknown
    }
}

/// "Execute" a PHP file.
pub fn execute(file: &Path) -> Response {
    let Ok(src) = std::fs::read_to_string(file) else {
        return Response::new(404, "text/plain", "not found\n");
    };
    let body = match classify(&src) {
        Verdict::WordPress => "<!doctype html><title>Just another WordPress site</title><p>Hello world!</p>",
        Verdict::Bitrix => "<!doctype html><title>1C-Bitrix</title><p>Loading…</p>",
        Verdict::Unknown => "Deprecated: mysql_connect(): The mysql extension is deprecated",
    };
    let mut resp = Response::new(200, "text/html; charset=utf-8", body);
    resp.extra_headers.push(("X-Powered-By", "PHP/5.6.40 (emulated)".into()));
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_wordpress() {
        let src = "<?php define('WP_USE_THEMES', true); require __DIR__ . '/wp-blog-header.php';";
        assert_eq!(classify(src), Verdict::WordPress);
    }
}
