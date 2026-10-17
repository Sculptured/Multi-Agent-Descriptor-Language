//! Open names (runtimes, mediums, protocols), durations, shorthand and extension keys.
use crate::diagnostic::{Diagnostic, Report};
use crate::duration;
use crate::model::{Document, Preferences, Settings};

pub const RUNTIMES: &[&str] = &[
    "claude-code",
    "codex",
    "camel",
    "crewai",
    "langgraph",
    "openai-agents",
    "autogen",
    "google-adk",
    "microsoft-agent-framework",
    "remote-a2a",
    "human",
];
pub const MEDIUMS: &[&str] = &["direct", "filesystem", "shared-memory", "blackboard", "bus", "git", "tracker", "human"];
const PROTOCOLS: &[&str] = &["native", "a2a", "mcp"];

/// `x-<vendor>-<name>`: lowercase letters, digits and `-`.
fn is_extension_name(name: &str) -> bool {
    name.strip_prefix("x-").is_some_and(|rest| {
        rest.contains('-') && rest.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

fn open_name(name: &str, known: &[&str]) -> bool {
    known.contains(&name) || is_extension_name(name)
}

/// `native`, `a2a@1.2`, `mcp@2025-06-18`, `x-acme-bus@2`.
pub fn is_protocol(text: &str) -> bool {
    let (name, version) = text.split_once('@').unwrap_or((text, ""));
    let version_ok = !text.contains('@')
        || (!version.is_empty() && version.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'));
    version_ok && open_name(name, PROTOCOLS)
}

pub fn check(doc: &Document, report: &mut Report) {
    for (i, a) in doc.agents.iter().enumerate() {
        agent(a, &format!("/agents/{i}"), report);
    }
    for (at, s) in settings_blocks(doc) {
        settings(s, &at, report);
    }
    channels_and_connections(doc, report);
    policies(doc, report);
    extension_keys(doc.extensions.keys(), "/extensions", report);
}

fn agent(a: &crate::model::Agent, at: &str, report: &mut Report) {
    runtime(a.runtime.as_deref(), &format!("{at}/runtime"), report);
    let prefs = a.settings.as_ref().and_then(|s| s.preferences.as_ref());
    let set = [a.runtime.is_some(), a.model.is_some(), a.resume.is_some()];
    shorthand(at, set, prefs, report);
    if a.runtime.as_deref() == Some("remote-a2a") && a.endpoint.is_none() {
        let why = format!("`{}` runs remotely but has no `endpoint`", a.id);
        report.push(Diagnostic::error("remote-needs-endpoint", at, why));
    }
    if let Some(t) = a.subagents.as_ref().and_then(|s| s.template.as_ref()) {
        runtime(t.runtime.as_deref(), &format!("{at}/subagents/template/runtime"), report);
    }
    extension_keys(a.extensions.keys(), &format!("{at}/extensions"), report);
}

fn runtime(name: Option<&str>, at: &str, report: &mut Report) {
    if let Some(n) = name.filter(|n| !open_name(n, RUNTIMES)) {
        report.push(Diagnostic::error(
            "unknown-runtime",
            at,
            format!("`{n}` is not a known runtime ({}) or an `x-<vendor>-<name>` extension", RUNTIMES.join(", ")),
        ));
    }
}

fn shorthand(at: &str, set: [bool; 3], prefs: Option<&Preferences>, report: &mut Report) {
    let Some(p) = prefs else { return };
    let long = [p.runtime.is_some(), p.model.is_some(), p.resume.is_some()];
    for (i, field) in ["runtime", "model", "resume"].into_iter().enumerate() {
        if set[i] && long[i] {
            let why = format!("`{field}` is set both here and in `settings.preferences`; keep one");
            report.push(Diagnostic::error("shorthand-conflict", format!("{at}/{field}"), why));
        }
    }
}

/// Every settings block with its pointer.
pub fn settings_blocks(doc: &Document) -> Vec<(String, &Settings)> {
    let mut out: Vec<(String, &Settings)> = doc.defaults.iter().map(|s| ("/defaults".to_string(), s)).collect();
    out.extend(
        doc.groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| Some((format!("/groups/{i}/settings"), g.settings.as_ref()?))),
    );
    for (i, a) in doc.agents.iter().enumerate() {
        out.extend(a.settings.iter().map(|s| (format!("/agents/{i}/settings"), s)));
        out.extend(
            a.subagents
                .iter()
                .filter_map(|s| s.settings.as_ref())
                .map(|s| (format!("/agents/{i}/subagents/settings"), s)),
        );
    }
    out
}

fn settings(s: &Settings, at: &str, report: &mut Report) {
    if let Some(p) = &s.preferences {
        runtime(p.runtime.as_deref(), &format!("{at}/preferences/runtime"), report);
        protocol(p.protocol.as_deref(), &format!("{at}/preferences/protocol"), report);
        let timeout = p.questions.as_ref().and_then(|q| q.timeout.as_deref());
        duration_of(timeout, &format!("{at}/preferences/questions/timeout"), report);
    }
    if let Some(b) = s.restrictions.as_ref().and_then(|r| r.budget.as_ref()) {
        super::limits::budget(b, &format!("{at}/restrictions/budget"), report);
    }
}

fn protocol(text: Option<&str>, at: &str, report: &mut Report) {
    if let Some(t) = text.filter(|t| !is_protocol(t)) {
        report.push(Diagnostic::error(
            "unknown-protocol",
            at,
            format!("`{t}` is not `native`, `a2a[@version]`, `mcp[@version]` or `x-<vendor>-<name>[@version]`"),
        ));
    }
}

pub(super) fn duration_of(text: Option<&str>, at: &str, report: &mut Report) {
    if let Some(t) = text.filter(|t| duration::seconds(t).is_none()) {
        report.push(Diagnostic::error(
            "bad-duration",
            at,
            format!("`{t}` is not a duration such as `90s`, `30m`, `2h` or `1h30m`"),
        ));
    }
}

fn channels_and_connections(doc: &Document, report: &mut Report) {
    for (i, c) in doc.channels.iter().enumerate() {
        if !open_name(&c.medium, MEDIUMS) {
            report.push(Diagnostic::error(
                "unknown-medium",
                format!("/channels/{i}/medium"),
                format!(
                    "`{}` is not a known medium ({}) or an `x-<vendor>-<name>` extension",
                    c.medium,
                    MEDIUMS.join(", ")
                ),
            ));
        }
        protocol(c.protocol.as_deref(), &format!("/channels/{i}/protocol"), report);
        extension_keys(c.extensions.keys(), &format!("/channels/{i}/extensions"), report);
    }
    for (i, c) in doc.connections.iter().enumerate() {
        protocol(c.protocol.as_deref(), &format!("/connections/{i}/protocol"), report);
        extension_keys(c.extensions.keys(), &format!("/connections/{i}/extensions"), report);
    }
}

fn policies(doc: &Document, report: &mut Report) {
    let Some(p) = &doc.policies else { return };
    for (i, s) in p.schedules.iter().enumerate() {
        duration_of(Some(&s.every), &format!("/policies/schedules/{i}/every"), report);
    }
    for (i, g) in p.guards.iter().enumerate() {
        duration_of(Some(&g.every), &format!("/policies/guards/{i}/every"), report);
    }
}

/// Reverse-DNS: at least two dot-separated labels of lowercase letters, digits and `-`.
fn extension_keys<'a>(keys: impl Iterator<Item = &'a String>, at: &str, report: &mut Report) {
    for key in keys {
        let labels: Vec<&str> = key.split('.').collect();
        let ok = labels.len() >= 2
            && labels
                .iter()
                .all(|l| !l.is_empty() && l.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        if !ok {
            report.push(Diagnostic::error(
                "bad-extension-key",
                format!("{at}/{key}"),
                format!("`{key}` is not a reverse-DNS namespace such as `com.example`"),
            ));
        }
    }
}
