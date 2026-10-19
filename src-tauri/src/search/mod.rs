use crate::domain::dto::{SearchHit, SearchInput, SearchResults};
use crate::domain::model::NodeType;
use crate::store::Store;
use std::collections::HashMap;
use std::time::Instant;

fn snippet(text: &str, query: &str) -> String {
    let lower = text.to_lowercase();
    let q = query.to_lowercase();
    let byte_index = if q.is_empty() { 0 } else { lower.find(&q).unwrap_or(0) };
    let char_index = text[..byte_index.min(text.len())].chars().count();
    let chars = text.chars().collect::<Vec<_>>();
    let start = char_index.saturating_sub(60);
    let end = (char_index + query.chars().count() + 120).min(chars.len());
    let mut out = chars[start..end].iter().collect::<String>().replace('\n', " ");
    if start > 0 {
        out = format!("…{out}");
    }
    if end < chars.len() {
        out.push('…');
    }
    out
}

pub fn search(store: &Store, input: &SearchInput) -> SearchResults {
    let started = Instant::now();
    let query = input.query.trim().to_lowercase();
    let limit = input.limit.unwrap_or(60).min(200);
    let type_filter = input.types.as_ref();
    let favorite_only = input.favorite_only.unwrap_or(false);

    let state = store.state().read();
    let project_alive: HashMap<String, bool> = state
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::Project)
        .map(|node| (node.id.clone(), node.is_live()))
        .collect();
    let project_names: HashMap<String, String> = state
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::Project)
        .map(|node| (node.id.clone(), node.title.clone()))
        .collect();

    let mut hits = Vec::new();
    for node in state.nodes.values() {
        if !node.is_live() {
            continue;
        }
        if favorite_only && !node.favorite {
            continue;
        }
        if node.node_type == NodeType::Run
            || node.node_type == NodeType::Settings
            || node.node_type == NodeType::ContextSnapshot
            || node.node_type == NodeType::Turn
            || node.node_type == NodeType::Chunk
            || node.node_type == NodeType::Attachment
            || node.node_type == NodeType::ModelProfile
            || node.node_type == NodeType::Folder
        {
            continue;
        }
        if let Some(project_id) = &node.project_id {
            if project_alive.get(project_id) == Some(&false) {
                continue;
            }
        }
        if let Some(filter) = type_filter {
            if !filter.is_empty() && !filter.iter().any(|t| t.as_str() == node.node_type.as_str()) {
                continue;
            }
        }
        if let Some(project_id) = &input.project_id {
            if node.project_id.as_deref() != Some(project_id.as_str()) {
                continue;
            }
        }

        let title = if node.title.is_empty() {
            node.body.lines().next().unwrap_or("").to_string()
        } else {
            node.title.clone()
        };
        let haystack = format!("{}\n{}", node.title, node.body).to_lowercase();
        if !query.is_empty() && !haystack.contains(&query) {
            continue;
        }
        if query.is_empty() && !favorite_only {
            // Empty query without a favorite filter is not a useful result set.
            continue;
        }
        let mut score = 1.0;
        if !query.is_empty() && node.title.to_lowercase().contains(&query) {
            score += 3.0;
        }
        if !query.is_empty() && node.body.to_lowercase().contains(&query) {
            score += 1.0;
        }
        if node.favorite {
            score += 0.5;
        }
        let card_id = if node.node_type == NodeType::Message {
            node.data_str("cardId").map(str::to_string)
        } else if node.node_type == NodeType::Card {
            Some(node.id.clone())
        } else {
            None
        };
        hits.push(SearchHit {
            node_id: node.id.clone(),
            node_type: node.node_type.as_str().into(),
            project_id: node.project_id.clone(),
            project_name: node
                .project_id
                .as_ref()
                .and_then(|id| project_names.get(id).cloned())
                .unwrap_or_default(),
            card_id,
            title,
            snippet: snippet(&node.body, &query),
            score,
            updated_at: node.updated_at,
        });
    }
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });
    hits.truncate(limit);
    SearchResults {
        query: input.query.clone(),
        hits,
        took_ms: started.elapsed().as_millis() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_snippet_does_not_split_a_character() {
        let text = "这是一个中文句子、包含标点。".repeat(20);
        let out = snippet(&text, "中文");
        assert!(out.contains("中文"));
    }
}
