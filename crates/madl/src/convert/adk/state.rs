//! ADK's session state as a shared-memory channel: agents write it through `output_key` and read
//! it through `{key}` in their instructions.
use crate::model::{Access, Channel};
use std::collections::BTreeMap;

/// Adds the channel when any agent writes or reads session state.
pub(super) fn channel(b: &mut crate::convert::Builder) {
    let access: BTreeMap<String, Access> =
        b.doc.agents.iter().filter_map(|a| Some((a.id.clone(), access_of(a)?))).collect();
    if access.is_empty() {
        return;
    }
    let id = b.id_for("session-state");
    let name = Some("ADK session state".to_string());
    b.doc.channels.push(Channel { id, name, medium: "shared-memory".into(), access, ..Default::default() });
}

fn access_of(a: &crate::model::Agent) -> Option<Access> {
    let writes = !a.outputs.is_empty();
    let reads = a.instructions.as_deref().is_some_and(|i| !references(i).is_empty());
    match (reads, writes) {
        (true, true) => Some(Access::ReadWrite),
        (true, false) => Some(Access::Read),
        (false, true) => Some(Access::Write),
        (false, false) => None,
    }
}

/// The state keys an instruction reads: `{key}`, `{key?}`, `{{key}}`, `{app:key}`,
/// `{artifact.name}`. Braces around anything else are left alone, as ADK leaves them.
pub(super) fn references(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, _) in text.match_indices('{') {
        let rest = text[i..].trim_start_matches('{');
        let Some(end) = rest.find('}') else { continue };
        let key = rest[..end].trim().trim_end_matches('?');
        let first_ok = key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_');
        let rest_ok = key.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.'));
        if first_ok && rest_ok && !out.iter().any(|k| k == key) {
            out.push(key.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::references;

    #[test]
    fn finds_state_keys_and_leaves_other_braces() {
        let text = "Code: {{current_document}} and {criticism} {notes?} {app:theme}. dict = {'a': 1} {}";
        assert_eq!(references(text), ["current_document", "criticism", "notes", "app:theme"]);
    }
}
