//! `kind: Prompt` agents as MADL agents.
use super::NS;
use super::load::File;
use crate::convert::{Builder, Fidelity};
use crate::model::{Agent, ModelRef, Preferences, Settings};
use serde_json::{Map, Value};

/// Keys of a Prompt agent that the document says in its own terms.
const KEYS: &[&str] =
    &["kind", "name", "displayName", "description", "instructions", "additionalInstructions", "model", "tools"];

/// The agent `file` defines, with id `id`.
pub(super) fn agent(b: &mut Builder, id: &str, file: &File) -> Agent {
    let at = format!("{}#", file.rel);
    let shown = file.text("displayName").or(file.text("name"));
    let mut agent = Agent {
        id: id.to_string(),
        name: shown.filter(|n| *n != id).map(str::to_string),
        description: file.text("description").map(str::to_string),
        instructions: instructions(b, file, &at),
        ..Default::default()
    };
    let mut ext = Map::new();
    if !file.value.contains_key("model") {
        b.note(Fidelity::Approximated, &at, "names no model: the chat client the host gives the agent decides it");
    }
    if let Some(model) = file.value.get("model") {
        agent.model = self::model(b, model, &format!("{at}/model")).map(ModelRef::Id);
        agent.settings = options(model);
        if let Some(c) = model.get("connection") {
            ext.insert("connection".into(), connection(b, c, &format!("{at}/model/connection")));
        }
    }
    let tool_args = super::tools::add(b, file, &mut agent);
    if !tool_args.is_empty() {
        ext.insert("tool_args".into(), Value::Object(tool_args));
    }
    ext.extend(leftovers(b, file, &at));
    if !ext.is_empty() {
        agent.extensions.insert(NS.into(), Value::Object(ext));
    }
    agent
}

/// `instructions`, then `additionalInstructions` after a blank line.
fn instructions(b: &mut Builder, file: &File, at: &str) -> Option<String> {
    let more = file.text("additionalInstructions");
    if more.is_some() {
        let why = "`additionalInstructions` were added after `instructions`, separated by a blank line";
        b.note(Fidelity::Approximated, format!("{at}/additionalInstructions"), why);
    }
    let parts: Vec<&str> = file.text("instructions").into_iter().chain(more).collect();
    (!parts.is_empty()).then(|| parts.join("\n\n"))
}

/// `provider/id`. A model read from the environment (`=Env.NAME`) has no fixed id.
fn model(b: &mut Builder, model: &Value, at: &str) -> Option<String> {
    let id = model.as_str().or_else(|| model.get("id")?.as_str()).map(str::trim).filter(|s| !s.is_empty())?;
    if let Some(expr) = id.strip_prefix('=') {
        let why = format!("the model is chosen when the agent loads (`{expr}`), so no model was set");
        b.note(Fidelity::Approximated, format!("{at}/id"), why);
        return None;
    }
    if id.contains('/') {
        return Some(id.to_string());
    }
    let Some(provider) = model.get("provider").and_then(Value::as_str) else {
        let why = "names no `provider`; the model was written without one, and the runner's client decides it";
        b.note(Fidelity::Approximated, at, why);
        return Some(id.to_string());
    };
    Some(format!("{}/{id}", provider_name(provider)))
}

/// Agent Framework's provider names as MADL providers.
fn provider_name(provider: &str) -> String {
    match provider.to_ascii_lowercase().as_str() {
        "azureopenai" => "azure-openai".into(),
        "azureai" | "azureaifoundry" | "foundry" => "azure-ai-foundry".into(),
        other => other.to_string(),
    }
}

/// `model.options` and `model.apiType` are the runtime's own options.
fn options(model: &Value) -> Option<Settings> {
    let mut options: std::collections::BTreeMap<String, Value> = model
        .get("options")
        .and_then(Value::as_object)
        .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    if let Some(api) = model.get("apiType") {
        options.insert("apiType".into(), api.clone());
    }
    let preferences = Preferences { options, ..Default::default() };
    (!preferences.options.is_empty()).then(|| Settings { preferences: Some(preferences), ..Default::default() })
}

/// How to reach the model, kept under `extensions`.
fn connection(b: &mut Builder, c: &Value, at: &str) -> Value {
    let why = format!("how to reach the model is kept under extensions `{NS}`, which other tools ignore");
    b.note(Fidelity::Unsupported, at, why);
    scrub(b, c.clone(), at)
}

/// `v` without the secret values written in it (`apiKey: sk-…`), at any depth: documents never
/// hold secret values. A value read from the environment (`=Env.KEY`) names no secret and stays.
pub(super) fn scrub(b: &mut Builder, v: Value, at: &str) -> Value {
    let Value::Object(m) = v else { return v };
    let literal = |v: &Value| v.as_str().is_some_and(|s| !s.trim_start().starts_with('='));
    let mut out = Map::new();
    for (k, v) in m {
        let here = format!("{at}/{k}");
        if is_secret(&k) && literal(&v) {
            b.note(Fidelity::Unsupported, here, format!("`{k}` holds a secret value; it was left out"));
        } else {
            out.insert(k, scrub(b, v, &here));
        }
    }
    Value::Object(out)
}

fn is_secret(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    ["key", "secret", "token", "password"].iter().any(|s| k.contains(s))
}

/// Keys with no MADL counterpart (`outputSchema`, `inputSchema`, `template`…): kept for the
/// Agent Framework under `extensions`, with a note.
fn leftovers(b: &mut Builder, file: &File, at: &str) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, value) in file.value.iter().filter(|(k, _)| !KEYS.contains(&k.as_str())) {
        let why =
            format!("`{key}` has no MADL counterpart; it is kept under extensions `{NS}`, which other tools ignore");
        b.note(Fidelity::Unsupported, format!("{at}/{key}"), why);
        let value = scrub(b, value.clone(), &format!("{at}/{key}"));
        out.insert(key.clone(), value);
    }
    out
}
