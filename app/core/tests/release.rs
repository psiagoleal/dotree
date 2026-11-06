// Caminho relativo: app/core/tests/release.rs
//! ENT-05 (MT-17): one version for the whole release. The crate version (shown by the site),
//! the web package, the README badge and the newest released section of the CHANGELOG
//! must agree, so a tag never publishes notes or a page that name another version.

use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn ent05_web_package_has_the_crate_version() {
    let package = repo_file("app/web/package.json");
    let line = format!("\"version\": \"{VERSION}\"");
    assert!(package.contains(&line), "package.json lacks {line}");
}

#[test]
fn ent05_readme_badge_has_the_crate_version() {
    let readme = repo_file("README.md");
    let badge = format!("badge/version-{VERSION}-blue");
    assert!(readme.contains(&badge), "README lacks {badge}");
}

#[test]
fn ent05_newest_changelog_release_is_the_crate_version_with_a_date() {
    let changelog = repo_file("CHANGELOG.md");
    let newest = changelog
        .lines()
        .filter(|l| l.starts_with("## ["))
        .find(|l| !l.starts_with("## [Não lançado]"))
        .expect("no released section");
    let prefix = format!("## [{VERSION}] - ");
    assert!(newest.starts_with(&prefix), "newest release is {newest:?}");
    let date = &newest[prefix.len()..];
    let parts: Vec<&str> = date.split('-').collect();
    assert!(
        parts.len() == 3 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())),
        "release date {date:?} is not YYYY-MM-DD"
    );
}

#[test]
fn ent05_released_section_is_not_empty() {
    let changelog = repo_file("CHANGELOG.md");
    let start = changelog
        .find(&format!("## [{VERSION}]"))
        .expect("section missing");
    let body = &changelog[start..];
    let body = &body[body.find('\n').unwrap()..];
    let end = body.find("\n## [").unwrap_or(body.len());
    assert!(
        body[..end].contains("\n### "),
        "the {VERSION} section has no entries"
    );
}
