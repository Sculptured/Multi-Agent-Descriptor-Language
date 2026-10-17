//! The Agent Framework importer on the files in `tests/maf/*/src` (see its README): the document
//! and the notes must match the checked-in files beside `src`, and the document must be valid.
//! Regenerate with `UPDATE_FIXTURES=1 cargo test`, then review every change by hand.
use madl::convert::maf;
use std::path::{Path, PathBuf};

fn folders() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/maf");
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
        let imported = maf::import(&dir.join("src")).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        let report = madl::validate(&imported.document);
        assert!(report.is_ok(), "{}: {:?}", dir.display(), report.errors().collect::<Vec<_>>());
        compare(&dir.join("expected.madl.yaml"), &serde_yaml_ng::to_string(&imported.document).unwrap());
        let notes: String = imported.notes.iter().map(|n| format!("{n}\n")).collect();
        compare(&dir.join("expected.notes"), &notes);
    }
}

#[test]
fn a_single_file_imports_like_a_folder_holding_it() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/maf/math_chat/src");
    let from_file = maf::import(&dir.join("MathChat.yaml")).unwrap();
    let from_folder = maf::import(&dir).unwrap();
    assert_eq!(from_file.document, from_folder.document);
    assert_eq!(from_file.notes, from_folder.notes);
}

#[test]
fn a_workflow_without_agents_is_an_error() {
    let dir = tempdir("no-agents");
    std::fs::write(dir.join("w.yaml"), "kind: Workflow\ntrigger:\n  kind: OnConversationStart\n  actions:\n    - kind: SendActivity\n      activity: hi\n").unwrap();
    let err = maf::import(&dir).unwrap_err();
    assert!(err.0.contains("invokes no agent"), "{err}");
}

#[test]
fn a_folder_with_two_workflows_is_an_error() {
    let dir = tempdir("two-workflows");
    for name in ["a.yaml", "b.yaml"] {
        std::fs::write(dir.join(name), "kind: Workflow\nactions: []\n").unwrap();
    }
    let err = maf::import(&dir).unwrap_err();
    assert!(err.0.contains("holds two workflows"), "{err}");
}

#[test]
fn a_file_of_another_kind_is_an_error() {
    let dir = tempdir("other-kind");
    std::fs::write(dir.join("x.yaml"), "kind: Skill\nname: x\n").unwrap();
    assert!(maf::import(&dir.join("x.yaml")).is_err());
    assert!(maf::import(&dir).is_err());
}

/// A fresh folder `<target tmp>/maf-<name>`.
fn tempdir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("maf-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
