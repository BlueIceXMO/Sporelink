use crate::domain::model::{Edge, Node, NodeType};
use crate::error::{AppError, AppResult};
use crate::store::Store;
use serde_json::{json, Value};
use std::path::Path;

const SECRET_KEYS: &[&str] = &[
    "apiKey",
    "api_key",
    "authorization",
    "Authorization",
    "secret",
    "password",
    "token",
];

pub fn sanitize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|key, _| !SECRET_KEYS.contains(&key.as_str()));
            for child in map.values_mut() {
                sanitize(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                sanitize(item);
            }
        }
        _ => {}
    }
}

fn sanitized_node(node: &Node) -> Value {
    let mut value = serde_json::to_value(node).unwrap_or(Value::Null);
    sanitize(&mut value);
    value
}

fn sanitized_edge(edge: &Edge) -> Value {
    let mut value = serde_json::to_value(edge).unwrap_or(Value::Null);
    sanitize(&mut value);
    value
}

pub fn vault_payload(store: &Store) -> Value {
    let nodes = store
        .all_nodes()
        .iter()
        .filter(|node| node.node_type != NodeType::Run)
        .map(sanitized_node)
        .collect::<Vec<_>>();
    let edges = store.all_edges().iter().map(sanitized_edge).collect::<Vec<_>>();
    json!({
        "format": "sporelink.vault",
        "schemaVersion": 1,
        "appVersion": "0.1.0-alpha",
        "exportedAt": crate::domain::model::now_ms(),
        "vault": { "id": "default", "name": "Sporelink Vault" },
        "nodes": nodes,
        "edges": edges,
        "sanitized": true,
    })
}

pub fn project_payload(store: &Store, project_id: &str) -> AppResult<Value> {
    let project = store
        .live_node(project_id)
        .filter(|node| node.node_type == NodeType::Project)
        .ok_or_else(|| AppError::not_found("project"))?;
    let mut nodes = vec![sanitized_node(&project)];
    nodes.extend(
        store
            .project_nodes(project_id)
            .iter()
            .filter(|node| node.node_type != NodeType::Run)
            .map(sanitized_node),
    );
    let edges = store
        .project_edges(project_id)
        .iter()
        .map(sanitized_edge)
        .collect::<Vec<_>>();
    Ok(json!({
        "format": "sporelink.project",
        "schemaVersion": 1,
        "appVersion": "0.1.0-alpha",
        "exportedAt": crate::domain::model::now_ms(),
        "projectId": project_id,
        "projectName": project.title,
        "nodes": nodes,
        "edges": edges,
        "sanitized": true,
    }))
}

pub fn write_json(path: &Path, value: &Value) -> AppResult<()> {
    let text = serde_json::to_string_pretty(value)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

pub fn read_json(path: &Path) -> AppResult<Value> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn project_markdown(store: &Store, project_id: &str) -> AppResult<String> {
    let project = store
        .live_node(project_id)
        .filter(|node| node.node_type == NodeType::Project)
        .ok_or_else(|| AppError::not_found("project"))?;
    let mut out = format!("# {}\n\n", project.title);
    let description = project.data_str("description").unwrap_or("");
    if !description.is_empty() {
        out.push_str(description);
        out.push_str("\n\n");
    }
    let cards = store
        .project_nodes(project_id)
        .into_iter()
        .filter(|node| node.node_type == NodeType::Card && node.is_live())
        .collect::<Vec<_>>();
    let mut cards = cards;
    cards.sort_by_key(|card| card.created_at);
    for card in cards {
        out.push_str(&format!("## {}\n\n", card.title));
        let mut turns = store
            .state()
            .read()
            .nodes
            .values()
            .filter(|node| {
                node.node_type == NodeType::Turn
                    && node.is_live()
                    && node.data_str("cardId") == Some(&card.id)
            })
            .cloned()
            .collect::<Vec<_>>();
        turns.sort_by_key(|turn| turn.data_i64("index").unwrap_or(0));
        let mut index = 1;
        for turn in turns {
            let mut messages = store
                .state()
                .read()
                .nodes
                .values()
                .filter(|node| {
                    node.node_type == NodeType::Message
                        && node.is_live()
                        && node.data_str("turnId") == Some(&turn.id)
                })
                .cloned()
                .collect::<Vec<_>>();
            messages.sort_by_key(|message| message.created_at);
            for message in messages {
                let role = if message.data_str("role") == Some("user") {
                    "You"
                } else {
                    "Sporelink"
                };
                out.push_str(&format!("### {index}. {role}\n\n{}\n\n", message.body));
                index += 1;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_removes_secret_keys_recursively() {
        let mut value = serde_json::json!({
            "apiKey": "sk-secret",
            "nested": { "authorization": "Bearer secret", "ok": 1 },
            "list": [{ "password": "p", "keep": true }]
        });
        sanitize(&mut value);
        let text = value.to_string();
        assert!(!text.contains("sk-secret"));
        assert!(!text.contains("Bearer secret"));
        assert!(!text.contains("\"p\""));
        assert!(text.contains("keep"));
    }
}
