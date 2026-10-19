use crate::domain::model::{now_ms, Edge, Node, NodeType};
use crate::domain::dto::Counts;
use crate::error::{AppError, AppResult};
use parking_lot::{Mutex, RwLock};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use velesdb_core::collection::MetadataCollection;
use velesdb_core::{Database, Point};

const NODES_COLLECTION: &str = "sporelink_nodes";
const EDGES_COLLECTION: &str = "sporelink_edges";

#[derive(Default)]
pub struct GraphState {
    pub nodes: HashMap<String, Node>,
    pub edges: HashMap<String, Edge>,
    pub node_points: HashMap<String, u64>,
    pub edge_points: HashMap<String, u64>,
    pub outgoing: HashMap<String, Vec<String>>,
    pub incoming: HashMap<String, Vec<String>>,
    pub next_node_point: u64,
    pub next_edge_point: u64,
}

impl GraphState {
    fn index_edge(&mut self, edge: &Edge) {
        self.outgoing
            .entry(edge.from.clone())
            .or_default()
            .push(edge.id.clone());
        self.incoming
            .entry(edge.to.clone())
            .or_default()
            .push(edge.id.clone());
    }

    fn unindex_edge(&mut self, edge: &Edge) {
        if let Some(v) = self.outgoing.get_mut(&edge.from) {
            v.retain(|id| id != &edge.id);
        }
        if let Some(v) = self.incoming.get_mut(&edge.to) {
            v.retain(|id| id != &edge.id);
        }
    }
}

pub struct Store {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    _db: Database,
    nodes: MetadataCollection,
    edges: MetadataCollection,
    state: RwLock<GraphState>,
    write_lock: Mutex<()>,
}

impl Store {
    pub fn open(data_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("db");
        std::fs::create_dir_all(&db_path)?;
        let db = Database::open(&db_path).map_err(|e| AppError::store(e.to_string()))?;

        if db.get_metadata_collection(NODES_COLLECTION).is_none() {
            db.create_metadata_collection(NODES_COLLECTION)
                .map_err(|e| AppError::store(e.to_string()))?;
        }
        if db.get_metadata_collection(EDGES_COLLECTION).is_none() {
            db.create_metadata_collection(EDGES_COLLECTION)
                .map_err(|e| AppError::store(e.to_string()))?;
        }

        let nodes = db
            .get_metadata_collection(NODES_COLLECTION)
            .ok_or_else(|| AppError::store("无法打开节点集合"))?;
        let edges = db
            .get_metadata_collection(EDGES_COLLECTION)
            .ok_or_else(|| AppError::store("无法打开边集合"))?;

        let state = load_graph(&nodes, &edges)?;
        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            db_path,
            _db: db,
            nodes,
            edges,
            state: RwLock::new(state),
            write_lock: Mutex::new(()),
        })
    }

    pub fn state(&self) -> &RwLock<GraphState> {
        &self.state
    }

    pub fn node(&self, id: &str) -> Option<Node> {
        self.state.read().nodes.get(id).cloned()
    }

    pub fn edge(&self, id: &str) -> Option<Edge> {
        self.state.read().edges.get(id).cloned()
    }

    pub fn live_node(&self, id: &str) -> Option<Node> {
        self.node(id).filter(Node::is_live)
    }

    pub fn all_nodes(&self) -> Vec<Node> {
        self.state.read().nodes.values().cloned().collect()
    }

    pub fn all_edges(&self) -> Vec<Edge> {
        self.state.read().edges.values().cloned().collect()
    }

    pub fn project_nodes(&self, project_id: &str) -> Vec<Node> {
        self.state
            .read()
            .nodes
            .values()
            .filter(|n| n.project_id.as_deref() == Some(project_id))
            .cloned()
            .collect()
    }

    pub fn project_edges(&self, project_id: &str) -> Vec<Edge> {
        self.state
            .read()
            .edges
            .values()
            .filter(|e| e.project_id.as_deref() == Some(project_id))
            .cloned()
            .collect()
    }

    pub fn outgoing_edges(&self, node_id: &str) -> Vec<Edge> {
        let state = self.state.read();
        state
            .outgoing
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| state.edges.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn incoming_edges(&self, node_id: &str) -> Vec<Edge> {
        let state = self.state.read();
        state
            .incoming
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| state.edges.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn put_node(&self, mut node: Node, expected_revision: Option<u64>) -> AppResult<Node> {
        let _guard = self.write_lock.lock();
        let mut state = self.state.write();

        let current = state.nodes.get(&node.id).cloned();
        if let Some(expected) = expected_revision {
            let actual = current.as_ref().map(|n| n.revision).unwrap_or(0);
            if expected != actual {
                return Err(AppError::revision_conflict(expected, actual));
            }
        }

        let point_id = match state.node_points.get(&node.id) {
            Some(id) => *id,
            None => {
                let id = state.next_node_point;
                state.next_node_point += 1;
                state.node_points.insert(node.id.clone(), id);
                id
            }
        };

        node.revision = current.as_ref().map(|n| n.revision + 1).unwrap_or(1);
        node.updated_at = now_ms();
        let payload = serde_json::to_value(&node)?;
        self.nodes
            .upsert(vec![Point::metadata_only(point_id, payload)])
            .map_err(|e| AppError::store(e.to_string()))?;
        self.nodes
            .flush()
            .map_err(|e| AppError::store(e.to_string()))?;

        state.nodes.insert(node.id.clone(), node.clone());
        Ok(node)
    }

    pub fn put_edge(&self, mut edge: Edge, expected_revision: Option<u64>) -> AppResult<Edge> {
        let _guard = self.write_lock.lock();
        let mut state = self.state.write();

        let current = state.edges.get(&edge.id).cloned();
        if let Some(expected) = expected_revision {
            let actual = current.as_ref().map(|e| e.revision).unwrap_or(0);
            if expected != actual {
                return Err(AppError::revision_conflict(expected, actual));
            }
        }

        let point_id = match state.edge_points.get(&edge.id) {
            Some(id) => *id,
            None => {
                let id = state.next_edge_point;
                state.next_edge_point += 1;
                state.edge_points.insert(edge.id.clone(), id);
                id
            }
        };

        if let Some(old) = current.as_ref() {
            state.unindex_edge(old);
        }
        edge.revision = current.as_ref().map(|e| e.revision + 1).unwrap_or(1);
        edge.updated_at = now_ms();
        let payload = serde_json::to_value(&edge)?;
        self.edges
            .upsert(vec![Point::metadata_only(point_id, payload)])
            .map_err(|e| AppError::store(e.to_string()))?;
        self.edges
            .flush()
            .map_err(|e| AppError::store(e.to_string()))?;

        state.edges.insert(edge.id.clone(), edge.clone());
        state.index_edge(&edge);
        Ok(edge)
    }

    /// Atomically enough for the alpha: validate all revisions first, then upsert.
    pub fn put_batch(&self, mut nodes: Vec<Node>, mut edges: Vec<Edge>) -> AppResult<()> {
        let _guard = self.write_lock.lock();
        let mut state = self.state.write();

        for node in nodes.iter_mut() {
            let current = state.nodes.get(&node.id).cloned();
            node.revision = current.as_ref().map(|n| n.revision + 1).unwrap_or(1);
            node.updated_at = now_ms();
        }
        for edge in edges.iter_mut() {
            let current = state.edges.get(&edge.id).cloned();
            edge.revision = current.as_ref().map(|e| e.revision + 1).unwrap_or(1);
            edge.updated_at = now_ms();
        }

        let mut node_points = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let point_id = match state.node_points.get(&node.id) {
                Some(id) => *id,
                None => {
                    let id = state.next_node_point;
                    state.next_node_point += 1;
                    state.node_points.insert(node.id.clone(), id);
                    id
                }
            };
            node_points.push(Point::metadata_only(
                point_id,
                serde_json::to_value(node)?,
            ));
        }
        let mut edge_points = Vec::with_capacity(edges.len());
        for edge in &edges {
            let point_id = match state.edge_points.get(&edge.id) {
                Some(id) => *id,
                None => {
                    let id = state.next_edge_point;
                    state.next_edge_point += 1;
                    state.edge_points.insert(edge.id.clone(), id);
                    id
                }
            };
            edge_points.push(Point::metadata_only(
                point_id,
                serde_json::to_value(edge)?,
            ));
        }

        if !node_points.is_empty() {
            self.nodes
                .upsert(node_points)
                .map_err(|e| AppError::store(e.to_string()))?;
            self.nodes
                .flush()
                .map_err(|e| AppError::store(e.to_string()))?;
        }
        if !edge_points.is_empty() {
            self.edges
                .upsert(edge_points)
                .map_err(|e| AppError::store(e.to_string()))?;
            self.edges
                .flush()
                .map_err(|e| AppError::store(e.to_string()))?;
        }

        for node in nodes {
            state.nodes.insert(node.id.clone(), node);
        }
        for edge in edges {
            if let Some(old) = state.edges.get(&edge.id).cloned() {
                state.unindex_edge(&old);
            }
            state.index_edge(&edge);
            state.edges.insert(edge.id.clone(), edge);
        }
        Ok(())
    }

    pub fn flush(&self) -> AppResult<()> {
        let _guard = self.write_lock.lock();
        self.nodes
            .flush()
            .map_err(|e| AppError::store(e.to_string()))?;
        self.edges
            .flush()
            .map_err(|e| AppError::store(e.to_string()))?;
        Ok(())
    }

    pub fn reload(&self) -> AppResult<()> {
        let _guard = self.write_lock.lock();
        let state = load_graph(&self.nodes, &self.edges)?;
        *self.state.write() = state;
        Ok(())
    }

    pub fn clear_all(&self) -> AppResult<()> {
        let _guard = self.write_lock.lock();
        let (node_ids, edge_ids) = {
            let state = self.state.read();
            (
                state.node_points.values().copied().collect::<Vec<_>>(),
                state.edge_points.values().copied().collect::<Vec<_>>(),
            )
        };
        if !node_ids.is_empty() {
            self.nodes
                .delete(&node_ids)
                .map_err(|e| AppError::store(e.to_string()))?;
        }
        if !edge_ids.is_empty() {
            self.edges
                .delete(&edge_ids)
                .map_err(|e| AppError::store(e.to_string()))?;
        }
        self.nodes
            .flush()
            .map_err(|e| AppError::store(e.to_string()))?;
        self.edges
            .flush()
            .map_err(|e| AppError::store(e.to_string()))?;
        *self.state.write() = GraphState::default();
        Ok(())
    }

    pub fn replace_all(&self, nodes: Vec<Node>, edges: Vec<Edge>) -> AppResult<()> {
        self.clear_all()?;
        self.put_batch(nodes, edges)
    }

    pub fn counts(&self) -> Counts {
        let state = self.state.read();
        let nodes = &state.nodes;
        Counts {
            projects: nodes
                .values()
                .filter(|n| n.node_type == NodeType::Project && n.is_live())
                .count(),
            cards: nodes
                .values()
                .filter(|n| n.node_type == NodeType::Card && n.is_live())
                .count(),
            messages: nodes
                .values()
                .filter(|n| n.node_type == NodeType::Message && n.is_live())
                .count(),
            documents: nodes
                .values()
                .filter(|n| n.node_type == NodeType::Document && n.is_live())
                .count(),
            knowledge: nodes
                .values()
                .filter(|n| n.node_type == NodeType::Knowledge && n.is_live())
                .count(),
            nodes: nodes.len(),
            edges: state.edges.len(),
        }
    }

    pub fn node_by_type(&self, node_type: NodeType) -> Vec<Node> {
        self.state
            .read()
            .nodes
            .values()
            .filter(|n| n.node_type == node_type)
            .cloned()
            .collect()
    }

    pub fn find_single_by_type(&self, node_type: NodeType) -> Option<Node> {
        self.state
            .read()
            .nodes
            .values()
            .find(|n| n.node_type == node_type)
            .cloned()
    }

    pub fn next_order(&self, parent_id: &str) -> i64 {
        self.outgoing_edges(parent_id)
            .iter()
            .filter(|e| e.relation == crate::domain::model::EdgeRelation::Contains)
            .map(|e| e.order)
            .max()
            .unwrap_or(-1)
            + 1
    }

    pub fn export_payload(&self) -> Value {
        let state = self.state.read();
        serde_json::json!({
            "nodes": state.nodes.values().collect::<Vec<_>>(),
            "edges": state.edges.values().collect::<Vec<_>>(),
        })
    }
}

fn load_graph(nodes: &MetadataCollection, edges: &MetadataCollection) -> AppResult<GraphState> {
    let mut state = GraphState::default();

    let mut cursor: Option<u64> = None;
    loop {
        let batch = nodes
            .scroll_batch(cursor, 512, None)
            .map_err(|e| AppError::store(e.to_string()))?;
        for point in batch.points {
            if let Some(value) = point.payload {
                match serde_json::from_value::<Node>(value) {
                    Ok(node) => {
                        state.node_points.insert(node.id.clone(), point.id);
                        state.next_node_point = state.next_node_point.max(point.id + 1);
                        state.nodes.insert(node.id.clone(), node);
                    }
                    Err(e) => {
                        tracing::warn!("skip malformed node point {}: {e}", point.id);
                    }
                }
            }
        }
        match batch.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    let mut cursor: Option<u64> = None;
    loop {
        let batch = edges
            .scroll_batch(cursor, 512, None)
            .map_err(|e| AppError::store(e.to_string()))?;
        for point in batch.points {
            if let Some(value) = point.payload {
                match serde_json::from_value::<Edge>(value) {
                    Ok(edge) => {
                        state.edge_points.insert(edge.id.clone(), point.id);
                        state.next_edge_point = state.next_edge_point.max(point.id + 1);
                        state.index_edge(&edge);
                        state.edges.insert(edge.id.clone(), edge);
                    }
                    Err(e) => {
                        tracing::warn!("skip malformed edge point {}: {e}", point.id);
                    }
                }
            }
        }
        match batch.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }

    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_roundtrip_survives_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let id;
        {
            let store = Store::open(dir.path()).expect("open");
            let node = Node::new(
                NodeType::Knowledge,
                None,
                "Test",
                "hello",
                serde_json::json!({"category": "insight"}),
            );
            id = node.id.clone();
            store.put_node(node, None).expect("put");
        }
        let store = Store::open(dir.path()).expect("reopen");
        let node = store.node(&id).expect("node");
        assert_eq!(node.title, "Test");
        assert_eq!(node.body, "hello");
    }
}
