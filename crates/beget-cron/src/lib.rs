//! CRON routing resolution.
//!
//! Upstream resolves a `Host:` header through several layers of nginx, apache and
//! a config generator written in a language we were unable to identify. We use a
//! directory lookup.

use std::path::{Component, Path, PathBuf};

/// Normalise a `Host:` header: strip port, lowercase, drop a leading `www.`.
pub fn normalize_host(raw: &str) -> String {
    let host = raw.trim().split(':').next().unwrap_or("").to_ascii_lowercase();
    host.strip_prefix("www.").unwrap_or(&host).to_string()
}

/// Map a host and request path onto a file under `sites_root`.
///
/// Returns `None` for anything that tries to escape the site directory.
pub fn resolve(sites_root: &Path, host: &str, req_path: &str) -> Option<PathBuf> {
    let host = normalize_host(host);
    if host.is_empty() || host.contains('/') || host.starts_with('.') {
        return None;
    }

    let path = req_path.split(['?', '#']).next().unwrap_or("/");
    let mut out = sites_root.join(&host).join("public_html");
    for comp in Path::new(path.trim_start_matches('/')).components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if path.ends_with('/') || out.is_dir() {
        out.push("index.html");
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_port_and_www() {
        assert_eq!(normalize_host("WWW.Example.ru:443"), "example.ru");
    }

    #[test]
    fn rejects_traversal() {
        assert!(resolve(Path::new("/srv"), "a.ru", "/../../etc/passwd").is_none());
    }

    #[test]
    fn maps_index() {
        let p = resolve(Path::new("/srv"), "a.ru", "/").unwrap();
        assert_eq!(p, PathBuf::from("/srv/a.ru/public_html/index.html"));
    }
}
