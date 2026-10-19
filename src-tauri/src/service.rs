use crate::context;
use crate::domain::dto::*;
use crate::domain::model::*;
use crate::error::{AppError, AppResult};
use crate::export;
use crate::keychain::Keychain;


use crate::state::{mock_profile, AppInner, SettingsCache};
use crate::store::Store;

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};


// ---------------------------------------------------------------------------
// Seed / settings
// ---------------------------------------------------------------------------

pub fn ensure_seed(store: &Store, _keychain: &Keychain) -> AppResult<(SettingsCache, Vec<ModelProfileDto>)> {
    let settings_node = if let Some(node) = store
        .find_single_by_type(NodeType::Settings)
        .filter(Node::is_live)
    {
        node
    } else {
        store.put_node(
            Node::new(
                NodeType::Settings,
                None,
                "Settings",
                "",
                serde_json::to_value(Settings::default())?,
            ),
            None,
        )?
    };

    let mut settings = serde_json::from_value::<Settings>(settings_node.data.clone())
        .unwrap_or_else(|_| Settings::default());

    let existing_mock = {
        let state = store.state().read();
        state
            .nodes
            .values()
            .find(|node| {
                node.node_type == NodeType::ModelProfile
                    && node.is_live()
                    && node.data_str("provider") == Some("mock")
            })
            .cloned()
    };
    let mock = if let Some(node) = existing_mock {
        node
    } else {
        store.put_node(
            Node::new(
                NodeType::ModelProfile,
                None,
                "Mock (offline)",
                "",
                serde_json::to_value(mock_profile())?,
            ),
            None,
        )?
    };

    if settings.model_assignments.chat.is_none() {
        settings.model_assignments.chat = Some(mock.id.clone());
    }
    if settings.model_assignments.summary.is_none() {
        settings.model_assignments.summary = Some(mock.id.clone());
    }
    if settings.model_assignments.title.is_none() {
        settings.model_assignments.title = Some(mock.id.clone());
    }
    let settings_node = if settings_node.data != serde_json::to_value(&settings)? {
        let mut node = settings_node.clone();
        node.data = serde_json::to_value(&settings)?;
        store.put_node(node, Some(settings_node.revision))?
    } else {
        settings_node
    };

    let models = list_model_profiles(store)?;
    Ok((
        SettingsCache {
            settings,
            node_id: settings_node.id,
            revision: settings_node.revision,
        },
        models,
    ))
}

pub fn list_model_profiles(store: &Store) -> AppResult<Vec<ModelProfileDto>> {
    let mut models = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::ModelProfile && node.is_live())
        .map(|node| {
            let data = serde_json::from_value::<ModelProfileData>(node.data.clone())
                .unwrap_or_else(|_| mock_profile());
            ModelProfileDto {
                id: node.id.clone(),
                revision: node.revision,
                data,
            }
        })
        .collect::<Vec<_>>();
    models.sort_by_key(|model| model.data.label.to_lowercase());
    Ok(models)
}

// ---------------------------------------------------------------------------
// Projects & folders
// ---------------------------------------------------------------------------

pub fn project_node(store: &Store, id: &str) -> AppResult<Node> {
    store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Project)
        .ok_or_else(|| AppError::not_found("project"))
}

pub fn root_card_id(store: &Store, project_id: &str) -> Option<String> {
    let state = store.state().read();
    let cards = state
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Card
                && node.is_live()
                && node.project_id.as_deref() == Some(project_id)
        })
        .collect::<Vec<_>>();
    for card in &cards {
        let has_card_parent = state
            .incoming
            .get(&card.id)
            .map(|edge_ids| {
                edge_ids.iter().any(|edge_id| {
                    state
                        .edges
                        .get(edge_id)
                        .map(|edge| {
                            edge.relation == EdgeRelation::Contains
                                && state
                                    .nodes
                                    .get(&edge.from)
                                    .map(|parent| parent.node_type == NodeType::Card)
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        if !has_card_parent {
            return Some(card.id.clone());
        }
    }
    cards.first().map(|card| card.id.clone())
}

pub fn project_summary(store: &Store, node: &Node) -> ProjectSummary {
    let cards = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|n| {
            n.node_type == NodeType::Card
                && n.is_live()
                && n.project_id.as_deref() == Some(&node.id)
        })
        .count();
    let messages = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|n| {
            n.node_type == NodeType::Message
                && n.is_live()
                && n.project_id.as_deref() == Some(&node.id)
        })
        .count();
    ProjectSummary {
        id: node.id.clone(),
        name: node.title.clone(),
        description: node.data_str("description").unwrap_or("").into(),
        color: node.data_str("color").unwrap_or("#8ee39a").into(),
        folder_id: node.data_str("folderId").map(str::to_string),
        favorite: node.favorite,
        archived: node.archived,
        deleted_at: node.deleted_at,
        created_at: node.created_at,
        updated_at: node.updated_at,
        revision: node.revision,
        root_card_id: root_card_id(store, &node.id),
        card_count: cards,
        message_count: messages,
    }
}

pub fn project_detail(store: &Store, id: &str) -> AppResult<ProjectDetail> {
    let node = project_node(store, id)?;
    let summary = project_summary(store, &node);
    let root = summary
        .root_card_id
        .clone()
        .ok_or_else(|| AppError::invalid("项目没有根卡片"))?;
    Ok(ProjectDetail {
        project: summary,
        root_card_id: root,
        default_model_profile_id: node.data_str("defaultModelProfileId").map(str::to_string),
        read_only: node.data_bool("readOnly").unwrap_or(false),
    })
}

pub fn list_projects(store: &Store, include_archived: bool) -> Vec<ProjectSummary> {
    let mut projects = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Project
                && node.is_live()
                && (include_archived || !node.archived)
        })
        .cloned()
        .collect::<Vec<_>>();
    projects.sort_by_key(|node| -node.updated_at);
    projects.iter().map(|node| project_summary(store, node)).collect()
}

pub fn create_project(store: &Store, input: ProjectCreateInput) -> AppResult<ProjectDetail> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::invalid("项目名称不能为空"));
    }
    let project = Node::new(
        NodeType::Project,
        None,
        name,
        "",
        json!({
            "description": input.description.unwrap_or_default(),
            "color": input.color.unwrap_or_else(|| "#8ee39a".into()),
            "folderId": input.folder_id,
            "defaultModelProfileId": null,
            "readOnly": false,
        }),
    );
    let root = Node::new(NodeType::Card, Some(project.id.clone()), name, "", json!({}));
    let edge = Edge::new(
        Some(project.id.clone()),
        &project.id,
        &root.id,
        EdgeRelation::Contains,
        0,
        json!({}),
    );
    let project_id = project.id.clone();
    store.put_batch(vec![project, root], vec![edge])?;
    project_detail(store, &project_id)
}

pub fn update_project(
    store: &Store,
    id: &str,
    patch: ProjectPatch,
    expected_revision: u64,
) -> AppResult<ProjectDetail> {
    let mut node = project_node(store, id)?;
    if let Some(name) = patch.name {
        if name.trim().is_empty() {
            return Err(AppError::invalid("项目名称不能为空"));
        }
        node.title = name.trim().to_string();
    }
    let data = node.data.as_object_mut().ok_or_else(|| AppError::invalid("项目数据损坏"))?;
    if let Some(description) = patch.description {
        data.insert("description".into(), json!(description));
    }
    if let Some(color) = patch.color {
        data.insert("color".into(), json!(color));
    }
    if let Some(folder) = patch.folder_id {
        data.insert("folderId".into(), json!(folder));
    }
    if let Some(default_model) = patch.default_model_profile_id {
        data.insert("defaultModelProfileId".into(), json!(default_model));
    }
    if let Some(read_only) = patch.read_only {
        data.insert("readOnly".into(), json!(read_only));
    }
    if let Some(favorite) = patch.favorite {
        node.favorite = favorite;
    }
    if let Some(archived) = patch.archived {
        node.archived = archived;
    }
    store.put_node(node, Some(expected_revision))?;
    project_detail(store, id)
}

pub fn delete_project(store: &Store, id: &str) -> AppResult<()> {
    let mut node = project_node(store, id)?;
    node.deleted_at = Some(now_ms());
    store.put_node(node, None)?;
    Ok(())
}

pub fn restore_project(store: &Store, id: &str) -> AppResult<ProjectDetail> {
    let mut node = store
        .node(id)
        .filter(|node| node.node_type == NodeType::Project)
        .ok_or_else(|| AppError::not_found("project"))?;
    node.deleted_at = None;
    store.put_node(node, None)?;
    project_detail(store, id)
}

pub fn duplicate_project(store: &Store, id: &str, name: Option<String>) -> AppResult<ProjectDetail> {
    let source = project_node(store, id)?;
    let nodes = store.project_nodes(id);
    let edges = store.project_edges(id);
    let mut id_map = HashMap::new();
    for node in &nodes {
        id_map.insert(node.id.clone(), new_id());
    }
    let new_project_id = id_map
        .get(&source.id)
        .cloned()
        .ok_or_else(|| AppError::invalid("项目节点缺失"))?;

    let mut new_nodes = Vec::new();
    for node in &nodes {
        let mut copy = node.clone();
        copy.id = id_map[&node.id].clone();
        copy.project_id = Some(new_project_id.clone());
        copy.revision = 1;
        copy.deleted_at = None;
        copy.created_at = now_ms();
        copy.updated_at = copy.created_at;
        if node.id == source.id {
            copy.title = name.clone().unwrap_or_else(|| format!("{} (copy)", source.title));
        }
        copy.data = remap_json(copy.data, &id_map);
        new_nodes.push(copy);
    }
    let mut new_edges = Vec::new();
    for edge in &edges {
        let mut copy = edge.clone();
        copy.id = new_id();
        copy.project_id = Some(new_project_id.clone());
        copy.from = id_map.get(&edge.from).cloned().unwrap_or(edge.from.clone());
        copy.to = id_map.get(&edge.to).cloned().unwrap_or(edge.to.clone());
        copy.metadata = remap_json(copy.metadata, &id_map);
        copy.revision = 1;
        copy.created_at = now_ms();
        copy.updated_at = copy.created_at;
        new_edges.push(copy);
    }
    store.put_batch(new_nodes, new_edges)?;
    project_detail(store, &new_project_id)
}

pub fn list_folders(store: &Store) -> Vec<FolderSummary> {
    let mut folders = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::Folder && node.is_live())
        .map(|node| FolderSummary {
            id: node.id.clone(),
            name: node.title.clone(),
            parent_id: node.data_str("parentId").map(str::to_string),
            order: node.data_i64("order").unwrap_or(0),
            created_at: node.created_at,
            updated_at: node.updated_at,
            revision: node.revision,
        })
        .collect::<Vec<_>>();
    folders.sort_by_key(|folder| (folder.order, folder.name.clone()));
    folders
}

pub fn create_folder(store: &Store, input: FolderCreateInput) -> AppResult<FolderSummary> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::invalid("文件夹名称不能为空"));
    }
    let order = list_folders(store).len() as i64;
    let node = store.put_node(
        Node::new(
            NodeType::Folder,
            None,
            name,
            "",
            json!({ "parentId": input.parent_id, "order": order }),
        ),
        None,
    )?;
    Ok(FolderSummary {
        id: node.id.clone(),
        name: node.title.clone(),
        parent_id: node.data_str("parentId").map(str::to_string),
        order: node.data_i64("order").unwrap_or(0),
        created_at: node.created_at,
        updated_at: node.updated_at,
        revision: node.revision,
    })
}

pub fn update_folder(
    store: &Store,
    id: &str,
    patch: FolderPatch,
    expected_revision: u64,
) -> AppResult<FolderSummary> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Folder)
        .ok_or_else(|| AppError::not_found("folder"))?;
    if let Some(name) = patch.name {
        if name.trim().is_empty() {
            return Err(AppError::invalid("文件夹名称不能为空"));
        }
        node.title = name.trim().to_string();
    }
    if let Some(parent) = patch.parent_id {
        if let Some(data) = node.data.as_object_mut() {
            data.insert("parentId".into(), json!(parent));
        }
    }
    let node = store.put_node(node, Some(expected_revision))?;
    Ok(FolderSummary {
        id: node.id.clone(),
        name: node.title.clone(),
        parent_id: node.data_str("parentId").map(str::to_string),
        order: node.data_i64("order").unwrap_or(0),
        created_at: node.created_at,
        updated_at: node.updated_at,
        revision: node.revision,
    })
}

pub fn delete_folder(store: &Store, id: &str) -> AppResult<()> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Folder)
        .ok_or_else(|| AppError::not_found("folder"))?;
    node.deleted_at = Some(now_ms());
    store.put_node(node, None)?;
    let mut projects = Vec::new();
    for project in store.state().read().nodes.values() {
        if project.node_type == NodeType::Project
            && project.is_live()
            && project.data_str("folderId") == Some(id)
        {
            let mut project = project.clone();
            if let Some(data) = project.data.as_object_mut() {
                data.insert("folderId".into(), Value::Null);
            }
            projects.push(project);
        }
    }
    if !projects.is_empty() {
        store.put_batch(projects, vec![])?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Graph & cards
// ---------------------------------------------------------------------------

pub fn card_view(store: &Store, node: &Node) -> CardView {
    let incoming = store.incoming_edges(&node.id);
    let outgoing = store.outgoing_edges(&node.id);
    let parent = incoming
        .iter()
        .find(|edge| edge.relation == EdgeRelation::Contains)
        .filter(|edge| {
            store
                .node(&edge.from)
                .map(|parent| parent.node_type == NodeType::Card)
                .unwrap_or(false)
        });
    let branch = outgoing
        .iter()
        .find(|edge| edge.relation == EdgeRelation::BranchOf)
        .or_else(|| {
            incoming
                .iter()
                .find(|edge| edge.relation == EdgeRelation::BranchOf)
        });
    let derives = outgoing
        .iter()
        .find(|edge| edge.relation == EdgeRelation::DerivesFrom)
        .or_else(|| {
            incoming
                .iter()
                .find(|edge| edge.relation == EdgeRelation::DerivesFrom)
        });
    let parallel = outgoing
        .iter()
        .find(|edge| edge.relation == EdgeRelation::ParallelTo)
        .or_else(|| {
            incoming
                .iter()
                .find(|edge| edge.relation == EdgeRelation::ParallelTo)
        });
    let source_message_id = branch
        .or(derives)
        .and_then(|edge| edge.metadata.get("sourceMessageId").and_then(Value::as_str))
        .map(str::to_string);
    let branch_point_message_id = branch
        .and_then(|edge| edge.metadata.get("contextCutMessageId").and_then(Value::as_str))
        .map(str::to_string);
    let source_card_id = parallel
        .map(|edge| {
            if edge.from == node.id {
                edge.to.clone()
            } else {
                edge.from.clone()
            }
        })
        .filter(|id| id != &node.id);
    let surface = if parent.is_none() {
        CardSurface::Root
    } else if parallel.is_some() {
        CardSurface::Diverge
    } else if branch.is_some() {
        CardSurface::Branch
    } else if derives.is_some() {
        CardSurface::Dive
    } else {
        CardSurface::Child
    };
    let child_count = outgoing
        .iter()
        .filter(|edge| {
            edge.relation == EdgeRelation::Contains
                && store
                    .node(&edge.to)
                    .map(|child| child.node_type == NodeType::Card && child.is_live())
                    .unwrap_or(false)
        })
        .count();
    let message_count = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|candidate| {
            candidate.node_type == NodeType::Message
                && candidate.is_live()
                && candidate.data_str("cardId") == Some(&node.id)
        })
        .count();
    CardView {
        id: node.id.clone(),
        title: node.title.clone(),
        surface_kind: surface,
        position: node.position,
        favorite: node.favorite,
        locked: node.locked,
        archived: node.archived,
        parent_id: parent.map(|edge| edge.from.clone()),
        source_card_id,
        source_message_id,
        branch_point_message_id,
        child_count,
        message_count,
        created_at: node.created_at,
        updated_at: node.updated_at,
        revision: node.revision,
    }
}

pub fn graph_view(store: &Store, project_id: &str) -> AppResult<GraphView> {
    let project = project_node(store, project_id)?;
    let _ = project;
    let cards = store
        .project_nodes(project_id)
        .into_iter()
        .filter(|node| node.node_type == NodeType::Card && node.is_live())
        .collect::<Vec<_>>();
    let card_ids = cards.iter().map(|card| card.id.clone()).collect::<HashSet<_>>();
    let edges = store
        .project_edges(project_id)
        .into_iter()
        .filter(|edge| {
            card_ids.contains(&edge.from)
                && card_ids.contains(&edge.to)
                && matches!(
                    edge.relation,
                    EdgeRelation::Contains
                        | EdgeRelation::ParallelTo
                        | EdgeRelation::BranchOf
                        | EdgeRelation::DerivesFrom
                )
        })
        .map(|edge| EdgeView {
            id: edge.id,
            from: edge.from,
            to: edge.to,
            relation: edge.relation.as_str().into(),
            order: edge.order,
            metadata: edge.metadata,
        })
        .collect::<Vec<_>>();
    let documents = store
        .project_nodes(project_id)
        .into_iter()
        .filter(|node| node.node_type == NodeType::Document && node.is_live())
        .map(|node| NodeRef {
            id: node.id,
            node_type: node.node_type,
            title: node.title,
            favorite: node.favorite,
            updated_at: node.updated_at,
        })
        .collect::<Vec<_>>();
    let knowledge = store
        .project_nodes(project_id)
        .into_iter()
        .filter(|node| node.node_type == NodeType::Knowledge && node.is_live())
        .map(|node| NodeRef {
            id: node.id,
            node_type: node.node_type,
            title: node.title,
            favorite: node.favorite,
            updated_at: node.updated_at,
        })
        .collect::<Vec<_>>();
    Ok(GraphView {
        project_id: project_id.into(),
        root_card_id: root_card_id(store, project_id),
        cards: cards.iter().map(|card| card_view(store, card)).collect(),
        edges,
        documents,
        knowledge,
    })
}

pub fn card_detail(store: &Store, id: &str) -> AppResult<CardDetail> {
    let node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    let card = card_view(store, &node);
    let mut path = Vec::new();
    let mut current = Some(card.clone());
    let mut guard = 0;
    while let Some(item) = current {
        path.push(CardPathItem {
            id: item.id.clone(),
            title: item.title.clone(),
            surface_kind: item.surface_kind,
        });
        if guard > 200 {
            break;
        }
        guard += 1;
        current = item
            .parent_id
            .as_ref()
            .and_then(|parent_id| store.live_node(parent_id))
            .filter(|parent| parent.node_type == NodeType::Card)
            .map(|parent| card_view(store, &parent));
    }
    path.reverse();
    let turns = turns_for_card(store, id)
        .into_iter()
        .map(|turn| TurnDto {
            id: turn.id.clone(),
            card_id: id.into(),
            index: turn.data_i64("index").unwrap_or(0),
            collapsed: turn.data_bool("collapsed").unwrap_or(false),
            model_profile_id: turn.data_str("modelProfileId").map(str::to_string),
            context_snapshot_id: turn.data_str("contextSnapshotId").map(str::to_string),
            messages: messages_for_turn(store, &turn.id)
                .iter()
                .map(|message| context::message_to_dto(store, message))
                .collect(),
            created_at: turn.created_at,
            updated_at: turn.updated_at,
            revision: turn.revision,
        })
        .collect();
    let project = node
        .project_id
        .as_ref()
        .and_then(|project_id| project_node(store, project_id).ok());
    Ok(CardDetail {
        card,
        path,
        turns,
        default_model_profile_id: project
            .as_ref()
            .and_then(|project| project.data_str("defaultModelProfileId").map(str::to_string)),
    })
}

fn turns_for_card(store: &Store, card_id: &str) -> Vec<Node> {
    let mut turns = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Turn
                && node.is_live()
                && node.data_str("cardId") == Some(card_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    turns.sort_by_key(|turn| turn.data_i64("index").unwrap_or(0));
    turns
}

fn messages_for_turn(store: &Store, turn_id: &str) -> Vec<Node> {
    let mut messages = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Message
                && node.is_live()
                && node.data_str("turnId") == Some(turn_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| {
        (
            message.data_i64("index").unwrap_or(i64::MAX),
            message.created_at,
        )
    });
    messages
}

pub fn create_card(store: &Store, input: CardCreateInput) -> AppResult<CardDetail> {
    let project = project_node(store, &input.project_id)?;
    if project.data_bool("readOnly").unwrap_or(false) {
        return Err(AppError::new("invalid_input", "项目只读"));
    }
    let parent = store
        .live_node(&input.parent_card_id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("parent card"))?;
    let source_message_id = input
        .source_message_id
        .clone()
        .or_else(|| last_assistant_message(store, &parent.id));
    let title = input
        .title
        .clone()
        .or_else(|| {
            source_message_id
                .as_ref()
                .and_then(|id| store.live_node(id))
                .map(|message| {
                    message
                        .body
                        .lines()
                        .find(|line| !line.trim().is_empty())
                        .map(|line| {
                            let cleaned = line
                                .trim()
                                .trim_start_matches('#')
                                .trim()
                                .trim_start_matches('*')
                                .trim();
                            let mut short = cleaned.chars().take(36).collect::<String>();
                            if cleaned.chars().count() > 36 {
                                short.push('…');
                            }
                            if short.is_empty() {
                                "New card".into()
                            } else {
                                short
                            }
                        })
                        .unwrap_or_else(|| "New card".into())
                })
        })
        .unwrap_or_else(|| "New card".into());

    let card = Node::new(
        NodeType::Card,
        Some(input.project_id.clone()),
        title,
        "",
        json!({}),
    );
    let mut nodes = vec![card.clone()];
    let mut edges = Vec::new();
    let relation = input.relation.as_str();
    let actual_parent = if relation == "diverge" {
        let parent_parent = store
            .incoming_edges(&parent.id)
            .into_iter()
            .find(|edge| edge.relation == EdgeRelation::Contains)
            .and_then(|edge| store.live_node(&edge.from))
            .filter(|node| node.node_type == NodeType::Card)
            .map(|node| node.id)
            .unwrap_or_else(|| parent.id.clone());
        parent_parent
    } else {
        parent.id.clone()
    };
    edges.push(Edge::new(
        Some(input.project_id.clone()),
        &actual_parent,
        &card.id,
        EdgeRelation::Contains,
        store.next_order(&actual_parent),
        json!({}),
    ));
    if let Some(source_message_id) = &source_message_id {
        let relation_edge = match relation {
            "branch" => Some(EdgeRelation::BranchOf),
            "dive" | "diverge" => Some(EdgeRelation::DerivesFrom),
            _ => None,
        };
        if let Some(relation_edge) = relation_edge {
            edges.push(Edge::new(
                Some(input.project_id.clone()),
                &card.id,
                source_message_id,
                relation_edge,
                0,
                json!({
                    "sourceMessageId": source_message_id,
                    "contextCutMessageId": source_message_id,
                }),
            ));
        }
    }
    if relation == "diverge" {
        edges.push(Edge::new(
            Some(input.project_id.clone()),
            &card.id,
            &parent.id,
            EdgeRelation::ParallelTo,
            0,
            json!({}),
        ));
    }
    store.put_batch(nodes.drain(..).collect(), edges)?;
    card_detail(store, &card.id)
}

fn last_assistant_message(store: &Store, card_id: &str) -> Option<String> {
    let mut messages = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Message
                && node.is_live()
                && node.data_str("cardId") == Some(card_id)
                && node.data_str("role") == Some("assistant")
        })
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| -message.created_at);
    messages.first().map(|message| message.id.clone())
}

pub fn update_card(
    store: &Store,
    id: &str,
    patch: CardPatch,
    expected_revision: u64,
) -> AppResult<CardDetail> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    if let Some(title) = patch.title {
        if !title.trim().is_empty() {
            node.title = title.trim().to_string();
        }
    }
    if let Some(position) = patch.position {
        node.position = Some(position);
    }
    if let Some(locked) = patch.locked {
        node.locked = locked;
    }
    if let Some(favorite) = patch.favorite {
        node.favorite = favorite;
    }
    if let Some(archived) = patch.archived {
        node.archived = archived;
    }
    store.put_node(node, Some(expected_revision))?;
    card_detail(store, id)
}

fn subtree_card_ids(store: &Store, id: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut stack = vec![id.to_string()];
    let mut seen = HashSet::new();
    while let Some(card_id) = stack.pop() {
        if !seen.insert(card_id.clone()) {
            continue;
        }
        result.push(card_id.clone());
        for edge in store.outgoing_edges(&card_id) {
            if edge.relation == EdgeRelation::Contains {
                if let Some(child) = store.live_node(&edge.to) {
                    if child.node_type == NodeType::Card {
                        stack.push(child.id);
                    }
                }
            }
        }
    }
    result
}

pub fn delete_card(store: &Store, id: &str, mode: &str) -> AppResult<()> {
    let card = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    if mode == "node" {
        let parent_edge = store
            .incoming_edges(id)
            .into_iter()
            .find(|edge| edge.relation == EdgeRelation::Contains);
        let parent_id = parent_edge.as_ref().map(|edge| edge.from.clone());
        if let Some(parent_id) = parent_id {
            let children = store
                .outgoing_edges(id)
                .into_iter()
                .filter(|edge| edge.relation == EdgeRelation::Contains)
                .filter(|edge| {
                    store
                        .live_node(&edge.to)
                        .map(|node| node.node_type == NodeType::Card)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            for mut edge in children {
                edge.from = parent_id.clone();
                edge.metadata = json!({ "reparentedFrom": id });
                store.put_edge(edge, None)?;
            }
        } else {
            // A root card with no parent: delete its subtree instead.
            return delete_card(store, id, "tree");
        }
    }
    let ids = if mode == "node" {
        vec![id.to_string()]
    } else {
        subtree_card_ids(store, id)
    };
    let mut nodes = Vec::new();
    for card_id in &ids {
        if let Some(mut card) = store.live_node(card_id) {
            card.deleted_at = Some(now_ms());
            nodes.push(card);
        }
        for turn in turns_for_card(store, card_id) {
            let mut turn = turn.clone();
            turn.deleted_at = Some(now_ms());
            let turn_id = turn.id.clone();
            nodes.push(turn);
            for mut message in messages_for_turn(store, &turn_id) {
                message.deleted_at = Some(now_ms());
                nodes.push(message);
            }
        }
    }
    let _ = card;
    store.put_batch(nodes, vec![])?;
    Ok(())
}

fn subtree_card_ids_all(store: &Store, id: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut stack = vec![id.to_string()];
    let mut seen = HashSet::new();
    while let Some(card_id) = stack.pop() {
        if !seen.insert(card_id.clone()) {
            continue;
        }
        result.push(card_id.clone());
        for edge in store.outgoing_edges(&card_id) {
            if edge.relation == EdgeRelation::Contains {
                if let Some(child) = store.node(&edge.to) {
                    if child.node_type == NodeType::Card {
                        stack.push(child.id);
                    }
                }
            }
        }
    }
    result
}

fn turns_for_card_all(store: &Store, card_id: &str) -> Vec<Node> {
    let mut turns = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Turn && node.data_str("cardId") == Some(card_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    turns.sort_by_key(|turn| turn.data_i64("index").unwrap_or(0));
    turns
}

fn messages_for_turn_all(store: &Store, turn_id: &str) -> Vec<Node> {
    let mut messages = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Message && node.data_str("turnId") == Some(turn_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| {
        (
            message.data_i64("index").unwrap_or(i64::MAX),
            message.created_at,
        )
    });
    messages
}

pub fn restore_card(store: &Store, id: &str) -> AppResult<CardDetail> {
    let ids = subtree_card_ids_all(store, id);
    let mut nodes = Vec::new();
    for card_id in &ids {
        if let Some(mut card) = store.node(card_id) {
            card.deleted_at = None;
            nodes.push(card);
        }
        for turn in turns_for_card_all(store, card_id) {
            let mut turn = turn.clone();
            turn.deleted_at = None;
            let turn_id = turn.id.clone();
            nodes.push(turn);
            for mut message in messages_for_turn_all(store, &turn_id) {
                message.deleted_at = None;
                nodes.push(message);
            }
        }
    }
    store.put_batch(nodes, vec![])?;
    card_detail(store, id)
}

pub fn move_card(store: &Store, id: &str, position: [f64; 3]) -> AppResult<CardDetail> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    node.position = Some(position);
    store.put_node(node, None)?;
    card_detail(store, id)
}

pub fn set_card_flag(
    store: &Store,
    id: &str,
    key: &str,
    value: bool,
) -> AppResult<CardDetail> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Card)
        .ok_or_else(|| AppError::not_found("card"))?;
    match key {
        "favorite" => node.favorite = value,
        "locked" => node.locked = value,
        _ => return Err(AppError::invalid("未知卡片标志")),
    }
    store.put_node(node, None)?;
    card_detail(store, id)
}

// ---------------------------------------------------------------------------
// Turns & messages
// ---------------------------------------------------------------------------

pub fn update_turn(store: &Store, id: &str, patch: TurnPatch, expected_revision: u64) -> AppResult<TurnDto> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Turn)
        .ok_or_else(|| AppError::not_found("turn"))?;
    if let Some(collapsed) = patch.collapsed {
        if let Some(data) = node.data.as_object_mut() {
            data.insert("collapsed".into(), json!(collapsed));
        }
    }
    let node = store.put_node(node, Some(expected_revision))?;
    Ok(TurnDto {
        id: node.id.clone(),
        card_id: node.data_str("cardId").unwrap_or("").into(),
        index: node.data_i64("index").unwrap_or(0),
        collapsed: node.data_bool("collapsed").unwrap_or(false),
        model_profile_id: node.data_str("modelProfileId").map(str::to_string),
        context_snapshot_id: node.data_str("contextSnapshotId").map(str::to_string),
        messages: messages_for_turn(store, &node.id)
            .iter()
            .map(|message| context::message_to_dto(store, message))
            .collect(),
        created_at: node.created_at,
        updated_at: node.updated_at,
        revision: node.revision,
    })
}

pub fn delete_turn(store: &Store, id: &str) -> AppResult<()> {
    let mut nodes = Vec::new();
    if let Some(mut turn) = store.live_node(id).filter(|node| node.node_type == NodeType::Turn) {
        turn.deleted_at = Some(now_ms());
        nodes.push(turn.clone());
        for mut message in messages_for_turn(store, &turn.id) {
            message.deleted_at = Some(now_ms());
            nodes.push(message);
        }
    }
    store.put_batch(nodes, vec![])?;
    Ok(())
}

pub fn restore_turn(store: &Store, id: &str) -> AppResult<TurnDto> {
    let mut nodes = Vec::new();
    if let Some(mut turn) = store.node(id).filter(|node| node.node_type == NodeType::Turn) {
        turn.deleted_at = None;
        nodes.push(turn.clone());
        for mut message in messages_for_turn_all(store, &turn.id) {
            message.deleted_at = None;
            nodes.push(message);
        }
    }
    store.put_batch(nodes, vec![])?;
    update_turn(store, id, TurnPatch::default(), store.node(id).map(|n| n.revision).unwrap_or(0))
}

pub fn update_message(
    store: &Store,
    id: &str,
    patch: MessagePatch,
    expected_revision: u64,
) -> AppResult<MessageDto> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Message)
        .ok_or_else(|| AppError::not_found("message"))?;
    if let Some(content) = patch.content {
        node.body = content;
        if let Some(data) = node.data.as_object_mut() {
            data.insert("status".into(), json!("complete"));
        }
    }
    if let Some(reasoning) = patch.reasoning {
        if let Some(data) = node.data.as_object_mut() {
            data.insert("reasoning".into(), json!(reasoning));
        }
    }
    if let Some(favorite) = patch.favorite {
        node.favorite = favorite;
    }
    let node = store.put_node(node, Some(expected_revision))?;
    Ok(context::message_to_dto(store, &node))
}

pub fn delete_message(store: &Store, id: &str) -> AppResult<()> {
    if let Some(mut node) = store.live_node(id).filter(|node| node.node_type == NodeType::Message) {
        node.deleted_at = Some(now_ms());
        store.put_node(node, None)?;
    }
    Ok(())
}

pub fn restore_message(store: &Store, id: &str) -> AppResult<MessageDto> {
    let mut node = store
        .node(id)
        .filter(|node| node.node_type == NodeType::Message)
        .ok_or_else(|| AppError::not_found("message"))?;
    node.deleted_at = None;
    let node = store.put_node(node, None)?;
    Ok(context::message_to_dto(store, &node))
}

pub fn set_message_favorite(store: &Store, id: &str, value: bool) -> AppResult<MessageDto> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Message)
        .ok_or_else(|| AppError::not_found("message"))?;
    node.favorite = value;
    let node = store.put_node(node, None)?;
    Ok(context::message_to_dto(store, &node))
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

pub fn context_preview(
    inner: &AppInner,
    input: ContextPreviewInput,
) -> AppResult<ContextPreview> {
    let (_, model) = inner.model_profile(&input.model_profile_id)?;
    let compiled = context::compile(
        &inner.store,
        &input.card_id,
        &input.draft_content,
        &model,
        &input.context,
    )?;
    Ok(context::preview_dto(&compiled, &input.model_profile_id))
}

// ---------------------------------------------------------------------------
// Knowledge
// ---------------------------------------------------------------------------

pub fn knowledge_list(store: &Store, project_id: Option<&str>, query: Option<&str>, tag: Option<&str>) -> Vec<KnowledgeDto> {
    let query = query.unwrap_or("").to_lowercase();
    let mut items = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Knowledge
                && node.is_live()
                && project_id.map(|id| node.project_id.as_deref() == Some(id)).unwrap_or(true)
                && (query.is_empty()
                    || node.title.to_lowercase().contains(&query)
                    || node.body.to_lowercase().contains(&query))
                && tag
                    .map(|tag| {
                        node.data
                            .get("tags")
                            .and_then(Value::as_array)
                            .map(|tags| tags.iter().any(|value| value.as_str() == Some(tag)))
                            .unwrap_or(false)
                    })
                    .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    items.sort_by_key(|node| -node.updated_at);
    items.iter().map(knowledge_dto).collect()
}

pub fn knowledge_dto(node: &Node) -> KnowledgeDto {
    KnowledgeDto {
        id: node.id.clone(),
        project_id: node.project_id.clone(),
        title: node.title.clone(),
        body: node.body.clone(),
        category: node.data_str("category").unwrap_or("insight").into(),
        confidence: node
            .data
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.7),
        tags: serde_json::from_value(
            node.data.get("tags").cloned().unwrap_or(json!([])),
        )
        .unwrap_or_default(),
        source_refs: serde_json::from_value(
            node.data.get("sourceRefs").cloned().unwrap_or(json!([])),
        )
        .unwrap_or_default(),
        created_at: node.created_at,
        updated_at: node.updated_at,
        revision: node.revision,
    }
}

pub fn knowledge_get(store: &Store, id: &str) -> AppResult<KnowledgeDto> {
    let node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Knowledge)
        .ok_or_else(|| AppError::not_found("knowledge"))?;
    Ok(knowledge_dto(&node))
}

pub fn knowledge_create(store: &Store, input: KnowledgeCreateInput) -> AppResult<KnowledgeDto> {
    if input.title.trim().is_empty() || input.body.trim().is_empty() {
        return Err(AppError::invalid("知识标题与正文不能为空"));
    }
    let node = Node::new(
        NodeType::Knowledge,
        input.project_id,
        input.title.trim(),
        input.body,
        json!({
            "category": input.category.unwrap_or_else(|| "insight".into()),
            "confidence": input.confidence.unwrap_or(0.7),
            "tags": input.tags.unwrap_or_default(),
            "sourceRefs": input.source_refs.unwrap_or_default(),
        }),
    );
    let node = store.put_node(node, None)?;
    Ok(knowledge_dto(&node))
}

pub fn knowledge_update(
    store: &Store,
    id: &str,
    patch: KnowledgePatch,
    expected_revision: u64,
) -> AppResult<KnowledgeDto> {
    let mut node = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Knowledge)
        .ok_or_else(|| AppError::not_found("knowledge"))?;
    if let Some(title) = patch.title {
        node.title = title;
    }
    if let Some(body) = patch.body {
        node.body = body;
    }
    if let Some(data) = node.data.as_object_mut() {
        if let Some(category) = patch.category {
            data.insert("category".into(), json!(category));
        }
        if let Some(confidence) = patch.confidence {
            data.insert("confidence".into(), json!(confidence));
        }
        if let Some(tags) = patch.tags {
            data.insert("tags".into(), json!(tags));
        }
        if let Some(source_refs) = patch.source_refs {
            data.insert("sourceRefs".into(), json!(source_refs));
        }
    }
    let node = store.put_node(node, Some(expected_revision))?;
    Ok(knowledge_dto(&node))
}

pub fn knowledge_delete(store: &Store, id: &str) -> AppResult<()> {
    if let Some(mut node) = store.live_node(id).filter(|node| node.node_type == NodeType::Knowledge) {
        node.deleted_at = Some(now_ms());
        store.put_node(node, None)?;
    }
    Ok(())
}

pub fn knowledge_from_message(
    store: &Store,
    input: KnowledgeFromMessageInput,
) -> AppResult<KnowledgeDto> {
    let message = store
        .live_node(&input.message_id)
        .filter(|node| node.node_type == NodeType::Message)
        .ok_or_else(|| AppError::not_found("message"))?;
    let project_id = message.project_id.clone();
    let title = input
        .title
        .unwrap_or_else(|| {
            input
                .selection
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("Knowledge")
                .chars()
                .take(48)
                .collect()
        });
    let knowledge = Node::new(
        NodeType::Knowledge,
        project_id.clone(),
        title,
        input.selection,
        json!({
            "category": "insight",
            "confidence": 0.7,
            "tags": [],
            "sourceRefs": [{
                "sourceType": "message",
                "sourceId": message.id,
                "title": message.title,
                "locator": null,
            }],
        }),
    );
    let edge = Edge::new(
        project_id,
        &knowledge.id,
        &message.id,
        EdgeRelation::DerivesFrom,
        0,
        json!({ "sourceMessageId": message.id }),
    );
    store.put_batch(vec![knowledge.clone()], vec![edge])?;
    Ok(knowledge_dto(&knowledge))
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

pub async fn project_summary_generate(
    inner: &AppInner,
    project_id: &str,
    model_profile_id: Option<String>,
) -> AppResult<KnowledgeDto> {
    let project = project_node(&inner.store, project_id)?;
    let cards = inner
        .store
        .project_nodes(project_id)
        .into_iter()
        .filter(|node| node.node_type == NodeType::Card && node.is_live())
        .collect::<Vec<_>>();
    let mut text = String::new();
    for card in &cards {
        text.push_str(&format!("## {}\n", card.title));
        for turn in turns_for_card(&inner.store, &card.id) {
            for message in messages_for_turn(&inner.store, &turn.id) {
                text.push_str(&format!(
                    "{}: {}\n",
                    if message.data_str("role") == Some("user") { "User" } else { "Assistant" },
                    message.body
                ));
            }
        }
        text.push('\n');
    }
    let model_id = model_profile_id.or_else(|| {
        inner
            .settings
            .read()
            .settings
            .model_assignments
            .summary
            .clone()
    });
    let (profile_node, model) = match model_id {
        Some(id) => inner.model_profile(&id)?,
        None => {
            let mock = inner
                .store
                .state()
                .read()
                .nodes
                .values()
                .find(|node| {
                    node.node_type == NodeType::ModelProfile
                        && node.data_str("provider") == Some("mock")
                })
                .cloned()
                .ok_or_else(|| AppError::new("model_not_configured", "没有可用的摘要模型"))?;
            let data = serde_json::from_value::<ModelProfileData>(mock.data.clone())
                .map_err(|e| AppError::parse(e.to_string()))?;
            (mock, data)
        }
    };
    let key = inner.api_key(&profile_node.id).ok().flatten();
    let messages = vec![context::ChatMessage {
        role: "user".into(),
        content: format!(
            "请把下面这个项目的探索过程整理成一条可引用的摘要。保留关键问题、主要分支、结论与下一步。\n\n{text}"
        ),
    }];
    let outcome = crate::llm::complete(&inner.http, &model, key.as_deref(), &messages, "summary").await?;
    let title = format!("{} · Summary", project.title);
    let knowledge = Node::new(
        NodeType::Knowledge,
        Some(project_id.into()),
        title,
        outcome.text,
        json!({
            "category": "summary",
            "confidence": 0.7,
            "tags": ["summary"],
            "sourceRefs": [{ "sourceType": "project", "sourceId": project_id, "title": project.title, "locator": null }],
        }),
    );
    let edge = Edge::new(
        Some(project_id.into()),
        &knowledge.id,
        project_id,
        EdgeRelation::Summarizes,
        0,
        json!({}),
    );
    inner.store.put_batch(vec![knowledge.clone()], vec![edge])?;
    Ok(knowledge_dto(&knowledge))
}

pub fn knowledge_check_prompt(store: &Store, id: &str) -> AppResult<String> {
    let node = knowledge_get(store, id)?;
    Ok(format!(
        "请检查下面这条知识条目是否清晰、准确、可引用。指出偏差、遗漏与一个可验证的下一步。\n\n标题：{}\n\n正文：\n{}",
        node.title, node.body
    ))
}

// ---------------------------------------------------------------------------
// Import / snapshot helpers
// ---------------------------------------------------------------------------

pub fn remap_json(value: Value, map: &HashMap<String, String>) -> Value {
    match value {
        Value::String(text) => Value::String(map.get(&text).cloned().unwrap_or(text)),
        Value::Array(items) => Value::Array(items.into_iter().map(|item| remap_json(item, map)).collect()),
        Value::Object(map_value) => Value::Object(
            map_value
                .into_iter()
                .map(|(key, value)| (key, remap_json(value, map)))
                .collect(),
        ),
        other => other,
    }
}

pub fn parse_import_payload(value: &Value) -> AppResult<(Vec<Node>, Vec<Edge>, Option<String>)> {
    let format = value.get("format").and_then(Value::as_str).unwrap_or("");
    if format != "sporelink.project" && format != "sporelink.vault" {
        return Err(AppError::invalid("不是 Sporelink 项目或 Vault 导出文件"));
    }
    let nodes_value = value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::invalid("导出文件缺少 nodes"))?;
    let edges_value = value
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::invalid("导出文件缺少 edges"))?;
    let nodes = nodes_value
        .iter()
        .map(|value| serde_json::from_value::<Node>(value.clone()).map_err(AppError::from))
        .collect::<AppResult<Vec<_>>>()?;
    let edges = edges_value
        .iter()
        .map(|value| serde_json::from_value::<Edge>(value.clone()).map_err(AppError::from))
        .collect::<AppResult<Vec<_>>>()?;
    let project_id = value
        .get("projectId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            nodes
                .iter()
                .find(|node| node.node_type == NodeType::Project)
                .map(|node| node.id.clone())
        });
    Ok((nodes, edges, project_id))
}

pub fn import_graph(store: &Store, value: &Value, folder_id: Option<String>) -> AppResult<ImportResult> {
    let (mut nodes, mut edges, project_id) = parse_import_payload(value)?;
    let existing = store.state().read().nodes.keys().cloned().collect::<HashSet<_>>();
    let needs_remap = nodes.iter().any(|node| existing.contains(&node.id));
    let mut id_map = HashMap::new();
    if needs_remap {
        for node in &nodes {
            id_map.insert(node.id.clone(), new_id());
        }
    }
    let mut project_count = 0;
    for node in &mut nodes {
        if node.node_type == NodeType::Project {
            project_count += 1;
        }
        if let Some(new_id) = id_map.get(&node.id) {
            node.id = new_id.clone();
        }
        node.revision = 1;
        node.deleted_at = None;
        node.data = remap_json(node.data.clone(), &id_map);
        if node.node_type == NodeType::Project {
            if let Some(folder_id) = &folder_id {
                if let Some(data) = node.data.as_object_mut() {
                    data.insert("folderId".into(), json!(folder_id));
                }
            }
        }
        if let Some(project_id) = &project_id {
            if node.node_type != NodeType::Project && node.project_id.as_deref() == Some(project_id) {
                node.project_id = Some(
                    id_map
                        .get(project_id)
                        .cloned()
                        .unwrap_or_else(|| project_id.clone()),
                );
            }
        }
    }
    for edge in &mut edges {
        if let Some(new_id) = id_map.get(&edge.id) {
            edge.id = new_id.clone();
        }
        edge.from = id_map.get(&edge.from).cloned().unwrap_or_else(|| edge.from.clone());
        edge.to = id_map.get(&edge.to).cloned().unwrap_or_else(|| edge.to.clone());
        edge.revision = 1;
        edge.metadata = remap_json(edge.metadata.clone(), &id_map);
        if let Some(project_id) = &project_id {
            if edge.project_id.as_deref() == Some(project_id) {
                edge.project_id = Some(
                    id_map
                        .get(project_id)
                        .cloned()
                        .unwrap_or_else(|| project_id.clone()),
                );
            }
        }
    }
    let node_count = nodes.len();
    let edge_count = edges.len();
    store.put_batch(nodes, edges)?;
    Ok(ImportResult {
        format: value
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("sporelink")
            .into(),
        imported_projects: project_count,
        imported_nodes: node_count,
        imported_edges: edge_count,
        remapped: needs_remap,
    })
}

pub fn project_import(store: &Store, path: &Path, folder_id: Option<String>) -> AppResult<ProjectDetail> {
    let value = export::read_json(path)?;
    let before = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::Project)
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();
    import_graph(store, &value, folder_id)?;
    let project = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::Project && node.is_live() && !before.contains(&node.id))
        .max_by_key(|node| node.created_at)
        .map(|node| node.id.clone())
        .ok_or_else(|| AppError::invalid("导入文件没有项目"))?;
    project_detail(store, &project)
}

pub fn trash_list(store: &Store) -> Vec<TrashItem> {
    let state = store.state().read();
    let project_names = state
        .nodes
        .values()
        .filter(|node| node.node_type == NodeType::Project)
        .map(|node| (node.id.clone(), node.title.clone()))
        .collect::<HashMap<_, _>>();
    let mut items = state
        .nodes
        .values()
        .filter(|node| {
            node.deleted_at.is_some()
                && matches!(
                    node.node_type,
                    NodeType::Project
                        | NodeType::Card
                        | NodeType::Turn
                        | NodeType::Message
                        | NodeType::Document
                        | NodeType::Knowledge
                )
        })
        .map(|node| TrashItem {
            id: node.id.clone(),
            node_type: node.node_type,
            title: if node.title.is_empty() {
                node.body.lines().next().unwrap_or("").chars().take(48).collect()
            } else {
                node.title.clone()
            },
            deleted_at: node.deleted_at.unwrap_or(0),
            project_id: node.project_id.clone(),
            project_name: node
                .project_id
                .as_ref()
                .and_then(|id| project_names.get(id).cloned())
                .unwrap_or_default(),
            card_id: node.data_str("cardId").map(str::to_string),
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| -item.deleted_at);
    items
}

pub fn trash_restore(store: &Store, id: &str) -> AppResult<()> {
    let node = store
        .node(id)
        .ok_or_else(|| AppError::not_found("trash item"))?;
    match node.node_type {
        NodeType::Project => {
            restore_project(store, id)?;
        }
        NodeType::Card => {
            restore_card(store, id)?;
        }
        NodeType::Turn => {
            restore_turn(store, id)?;
        }
        NodeType::Message => {
            restore_message(store, id)?;
        }
        NodeType::Knowledge => {
            let mut node = node;
            node.deleted_at = None;
            store.put_node(node, None)?;
        }
        NodeType::Document => {
            let mut nodes = Vec::new();
            let mut document = node;
            document.deleted_at = None;
            let document_id = document.id.clone();
            nodes.push(document);
            for chunk in store.state().read().nodes.values() {
                if chunk.node_type == NodeType::Chunk
                    && chunk.data_str("documentId") == Some(&document_id)
                    && chunk.deleted_at.is_some()
                {
                    let mut chunk = chunk.clone();
                    chunk.deleted_at = None;
                    nodes.push(chunk);
                }
            }
            store.put_batch(nodes, vec![])?;
        }
        _ => return Err(AppError::unsupported("这个对象不支持从回收站恢复")),
    }
    Ok(())
}

pub fn snapshot_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("vaults").join("default").join("snapshots")
}pub fn snapshot_create(data_dir: &Path, store: &Store, label: Option<String>) -> AppResult<SnapshotSummary> {
    let dir = snapshot_dir(data_dir);
    std::fs::create_dir_all(&dir)?;
    let id = new_id();
    let created_at = now_ms();
    let payload = export::vault_payload(store);
    let path = dir.join(format!("{id}.json"));
    export::write_json(&path, &payload)?;
    let node_count = payload.get("nodes").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
    let edge_count = payload.get("edges").and_then(Value::as_array).map(|a| a.len()).unwrap_or(0);
    let summary = SnapshotSummary {
        id: id.clone(),
        label: label.unwrap_or_else(|| format!("Snapshot {created_at}")),
        created_at,
        node_count,
        edge_count,
        path: path.to_string_lossy().to_string(),
    };
    let manifest = serde_json::to_string_pretty(&summary)?;
    std::fs::write(dir.join(format!("{id}.meta.json")), manifest)?;
    // Keep only the 10 most recent snapshots.
    let mut files = std::fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".meta.json"))
        .collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.metadata().and_then(|m| m.modified()).ok());
    if files.len() > 10 {
        for entry in files.iter().take(files.len() - 10) {
            let path = entry.path();
            let _ = std::fs::remove_file(&path);
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let id = stem.trim_end_matches(".meta");
                let _ = std::fs::remove_file(dir.join(format!("{id}.json")));
            }
        }
    }
    Ok(summary)
}

pub fn snapshot_list(data_dir: &Path) -> Vec<SnapshotSummary> {
    let dir = snapshot_dir(data_dir);
    let mut snapshots = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(".meta.json"))
                .filter_map(|entry| {
                    let text = std::fs::read_to_string(entry.path()).ok()?;
                    serde_json::from_str::<SnapshotSummary>(&text).ok()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    snapshots.sort_by_key(|snapshot| -snapshot.created_at);
    snapshots
}

pub fn snapshot_restore(data_dir: &Path, store: &Store, id: &str) -> AppResult<()> {
    let path = snapshot_dir(data_dir).join(format!("{id}.json"));
    let value = export::read_json(&path)?;
    let (nodes, edges, _) = parse_import_payload(&value)?;
    store.replace_all(nodes, edges)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_root_and_child_from_graph() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(dir.path()).expect("store");
        let project = create_project(
            &store,
            ProjectCreateInput {
                name: "Test".into(),
                description: None,
                color: None,
                folder_id: None,
            },
        )
        .expect("project");
        let root = card_detail(&store, &project.root_card_id).expect("root");
        assert_eq!(root.card.surface_kind, CardSurface::Root);
        let child = create_card(
            &store,
            CardCreateInput {
                project_id: project.project.id.clone(),
                parent_card_id: root.card.id.clone(),
                relation: "child".into(),
                source_message_id: None,
                title: Some("Child".into()),
            },
        )
        .expect("child");
        assert_eq!(child.card.surface_kind, CardSurface::Child);
        let payload = export::project_payload(&store, &project.project.id).expect("payload");
        assert!(payload["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .any(|node| node["nodeType"] == "project"));
    }
}
