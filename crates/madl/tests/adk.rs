//! The ADK importer on the configs in `tests/adk/` (see its README): the document and the notes
//! must match the checked-in files, and the document must be valid. Regenerate with
//! `UPDATE_FIXTURES=1 cargo test`, then review every change by hand.
use madl::convert::adk;
use std::path::{Path, PathBuf};

fn folders() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/adk");
    let mut out: Vec<PathBuf> =
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_dir()).collect();
    out.sort();
    out
}

fn compare(path: &Path, text: &str) {
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::write(path, text).unwrap();
    }
    let on_disk = std::fs::read_to_string(path).unwrap_or_default();
    assert!(on_disk == text, "{} differs from the importer:\n{text}", path.display());
}

#[test]
fn imports_match_the_fixtures() {
    for dir in folders() {
        let imported = adk::import(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let report = madl::validate(&imported.document);
        assert!(report.is_ok(), "{}: {:?}", dir.display(), report.errors().collect::<Vec<_>>());
        compare(&dir.join("expected.madl.yaml"), &serde_yaml_ng::to_string(&imported.document).unwrap());
        let notes: String = imported.notes.iter().map(|n| format!("{n}\n")).collect();
        compare(&dir.join("expected.notes"), &notes);
    }
}

#[test]
fn a_config_path_may_not_leave_the_folder() {
    let dir = tempdir("outside");
    std::fs::write(dir.join("root_agent.yaml"), "name: a\ninstruction: x\nsub_agents: [{ config_path: ../b.yaml }]\n")
        .unwrap();
    std::fs::write(dir.parent().unwrap().join("b.yaml"), "name: b\ninstruction: y\n").unwrap();
    let err = adk::import(&dir).unwrap_err();
    assert!(err.0.contains("is outside"), "{err}");
}

#[test]
fn a_missing_root_is_an_error() {
    let dir = tempdir("missing");
    assert!(adk::import(&dir).is_err());
}

/// A fresh folder `<target tmp>/adk-<name>/team`.
fn tempdir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adk-{name}")).join("team");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
