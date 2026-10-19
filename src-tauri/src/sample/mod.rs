use crate::domain::model::{now_ms, Edge, EdgeRelation, Node, NodeType};
use crate::error::AppResult;
use crate::store::Store;
use serde_json::json;

fn msg(
    project_id: &str,
    card_id: &str,
    turn_id: &str,
    role: &str,
    content: &str,
    reasoning: &str,
    offset: i64,
) -> Node {
    let mut node = Node::new(
        NodeType::Message,
        Some(project_id.into()),
        if role == "user" { "You" } else { "Sporelink" },
        content,
        json!({
            "role": role,
            "status": "complete",
            "reasoning": reasoning,
            "citations": [],
            "attachments": [],
            "turnId": turn_id,
            "cardId": card_id,
            "usage": null,
        }),
    );
    node.created_at = now_ms() + offset;
    node.updated_at = node.created_at;
    node
}

fn turn(
    project_id: &str,
    card_id: &str,
    index: i64,
    model_profile_id: Option<&str>,
    offset: i64,
) -> Node {
    let mut node = Node::new(
        NodeType::Turn,
        Some(project_id.into()),
        format!("Round {}", index + 1),
        "",
        json!({
            "cardId": card_id,
            "index": index,
            "collapsed": index > 0,
            "modelProfileId": model_profile_id,
            "contextSnapshotId": null,
        }),
    );
    node.created_at = now_ms() + offset;
    node.updated_at = node.created_at;
    node
}

fn card(project_id: &str, title: &str, offset: i64) -> Node {
    let mut node = Node::new(NodeType::Card, Some(project_id.into()), title, "", json!({}));
    node.created_at = now_ms() + offset;
    node.updated_at = node.created_at;
    node
}

pub fn install_sample(store: &Store, locale: &str) -> AppResult<String> {
    let en = locale == "en";
    let tw = locale == "zh-TW";
    let project_title = if en {
        "How to Use Sporelink"
    } else if tw {
        "如何使用 Sporelink"
    } else {
        "如何使用 Sporelink"
    };
    let project = Node::new(
        NodeType::Project,
        None,
        project_title,
        "",
        json!({
            "description": if en {
                "An original sample project: cards, rounds, context, knowledge, and export."
            } else {
                "原创示例项目：卡片树、轮次、上下文、知识与导出。"
            },
            "color": "#8ee39a",
            "folderId": null,
            "defaultModelProfileId": null,
            "readOnly": false,
        }),
    );
    let project_id = project.id.clone();

    let root = card(&project_id, if en { "Welcome" } else { "欢迎" }, 10);
    let dive = card(&project_id, if en { "How context is compiled" } else { "上下文是怎么编译的" }, 20);
    let diverge = card(&project_id, if en { "Another way to take notes" } else { "另一种记笔记的方式" }, 30);
    let branch = card(&project_id, if en { "If I only want a linear chat" } else { "如果我只想要线性对话" }, 40);

    let mut nodes = vec![project, root.clone(), dive.clone(), diverge.clone(), branch.clone()];
    let mut edges = vec![Edge::new(
        Some(project_id.clone()),
        &project_id,
        &root.id,
        EdgeRelation::Contains,
        0,
        json!({}),
    )];
    for item in [&dive, &diverge, &branch] {
        edges.push(Edge::new(
            Some(project_id.clone()),
            &root.id,
            &item.id,
            EdgeRelation::Contains,
            edges.len() as i64,
            json!({}),
        ));
    }

    let turn1 = turn(&project_id, &root.id, 0, None, 100);
    let m1 = msg(
        &project_id,
        &root.id,
        &turn1.id,
        "user",
        if en {
            "What is different between Sporelink and a normal chat?"
        } else {
            "Sporelink 和普通聊天有什么不同？"
        },
        "",
        101,
    );
    let m2 = msg(
        &project_id,
        &root.id,
        &turn1.id,
        "assistant",
        if en {
            "A normal chat is linear. Sporelink keeps the **structure** of the conversation.\n\n- Every card is a node; every relation is an edge.\n- A deep dive, a divergence, and a branch are not stored as mutually exclusive types. They are **derived from graph topology**.\n- Before sending, you can inspect the compiled context: which messages, citations, knowledge, and document chunks the model will actually receive.\n- Everything is local and exportable. The model key only lives in the OS keychain.\n\n> Try selecting a paragraph, then use the floating toolbar to quote it or save it as knowledge.\n\nThis sample is original content written for Sporelink. It is not copied from the reference product."
                .into()
        } else if tw {
            "一般聊天是線性的。Sporelink 保留的是對話的**結構**。\n\n- 每張卡片是節點，每段關係是邊。\n- 深潛、發散、分支不是互斥的「卡片類型」，而是**由圖拓撲派生**出來的表層標籤。\n- 發送前可以檢查編譯後的上下文：模型實際會收到哪些訊息、引用、知識與文件片段。\n- 一切都在本地，可完整匯出。模型 Key 只存在 OS keychain。\n\n> 試著選中一段文字，用浮動工具列引用它，或存為知識。\n\n這個示例是為 Sporelink 原創撰寫的，不複製參考產品。"
                .into()
        } else {
            "普通聊天是线性的。Sporelink 保留的是对话的**结构**。\n\n- 每张卡片是节点，每段关系是边。\n- 深潜、发散、分支不是互斥的“卡片类型”，而是**由图拓扑派生**出来的表层标签。\n- 发送前可以检查编译后的上下文：模型实际会收到哪些消息、引用、知识、文档片段。\n- 一切都在本地，可完整导出。模型 Key 只存在 OS keychain。\n\n> 试着选中一段文字，用浮动工具栏引用它，或存为知识。\n\n这个示例是为 Sporelink 原创撰写的，不复制参考产品。"
                .into()
        },
        if en {
            "The user wants to know the core difference. I should explain the node/edge model, then point to the context compiler and local-first boundaries."
        } else {
            "用户想知道核心差异。我会先解释节点/边模型，再指向上下文编译与本地优先边界。"
        },
        102,
    );
    let turn2 = turn(&project_id, &root.id, 1, None, 200);
    let m3 = msg(
        &project_id,
        &root.id,
        &turn2.id,
        "user",
        if en { "How should I start?" } else { "我该怎么开始？" },
        "",
        201,
    );
    let m4 = msg(
        &project_id,
        &root.id,
        &turn2.id,
        "assistant",
        if en {
            "1. Write a real question in the composer and send it.\n2. Select a useful paragraph and use **Deep dive** to create a child card.\n3. Use **Diverge** to explore a neighbouring direction without losing the main path.\n4. Open **Context** before sending to see the compiled input.\n5. Save the best insight as knowledge, then export the project.\n\nNo account, no telemetry, no cloud sync."
                .into()
        } else if tw {
            "1. 在輸入區寫下一個真正的問題並送出。\n2. 選中一段有用的文字，用**深潛**建立子卡片。\n3. 用**發散**探索相鄰方向，不丟失主路徑。\n4. 送出前打開**上下文**，查看編譯後的輸入。\n5. 把最好的理解存為知識，然後匯出專案。\n\n沒有帳號、沒有遙測、沒有雲同步。"
                .into()
        } else {
            "1. 在输入区写下一个真正的问题并发送。\n2. 选中一段有用的文字，用**深潜**创建子卡片。\n3. 用**发散**探索相邻方向，不丢失主路径。\n4. 发送前打开**上下文**，查看编译后的输入。\n5. 把最好的理解存为知识，然后导出项目。\n\n没有账号、没有遥测、没有云同步。"
                .into()
        },
        if en {
            "Give a short, actionable checklist. Keep the local-first boundary visible."
        } else {
            "给一个简短、可执行的清单，同时把本地优先边界说清楚。"
        },
        202,
    );
    let turn3 = turn(&project_id, &root.id, 2, None, 300);
    let m5 = msg(
        &project_id,
        &root.id,
        &turn3.id,
        "user",
        if en { "How does the card tree work?" } else { "卡片树怎么用？" },
        "",
        301,
    );
    let m6 = msg(
        &project_id,
        &root.id,
        &turn3.id,
        "assistant",
        if en {
            "- **Root**: the first card of a project.\n- **Deep dive**: continue from a selected message; the path is preserved up to that message.\n- **Diverge**: explore side by side; layout uses the `parallel_to` edge.\n- **Branch**: try a parallel answer from the same history; the context is cut at the branch point.\n\nSwitch cards from the 3D tree on the right. Drag to rotate, scroll to zoom, click a node to open it.\n\nMath works too: $e^{i\\pi} + 1 = 0$."
                .into()
        } else if tw {
            "- **根卡片**：專案的起點。\n- **深潛**：從選中的訊息繼續，保留到該訊息為止的路徑。\n- **發散**：並排探索，佈局使用 `parallel_to` 邊。\n- **分支**：從同一段歷史嘗試平行回答，上下文在分支點截斷。\n\n在右側 3D 卡片樹切換卡片。拖曳旋轉、滾輪縮放、點擊節點打開。\n\n數學公式也可以：$e^{i\\pi} + 1 = 0$。"
                .into()
        } else {
            "- **根卡片**：项目的起点。\n- **深潜**：从选中的消息继续，保留到该消息为止的路径。\n- **发散**：并排探索，布局使用 `parallel_to` 边。\n- **分支**：从同一段历史尝试平行回答，上下文在分支点截断。\n\n在右侧 3D 卡片树切换卡片。拖拽旋转、滚轮缩放、点击节点打开。\n\n数学公式也可以：$e^{i\\pi} + 1 = 0$。"
                .into()
        },
        if en {
            "Explain the three derived card surfaces as graph topology, not stored card types."
        } else {
            "把三种表层卡片解释为图拓扑，而不是落库的卡片类型。"
        },
        302,
    );

    for (turn_node, messages) in [
        (turn1.clone(), vec![m1.clone(), m2.clone()]),
        (turn2.clone(), vec![m3.clone(), m4.clone()]),
        (turn3.clone(), vec![m5.clone(), m6.clone()]),
    ] {
        edges.push(Edge::new(
            Some(project_id.clone()),
            &root.id,
            &turn_node.id,
            EdgeRelation::Contains,
            turn_node.data_i64("index").unwrap_or(0),
            json!({}),
        ));
        nodes.push(turn_node.clone());
        for (index, message) in messages.into_iter().enumerate() {
            edges.push(Edge::new(
                Some(project_id.clone()),
                &turn_node.id,
                &message.id,
                EdgeRelation::Contains,
                index as i64,
                json!({}),
            ));
            nodes.push(message);
        }
    }

    // Dive card, derived from the first assistant message.
    edges.push(Edge::new(
        Some(project_id.clone()),
        &dive.id,
        &m2.id,
        EdgeRelation::DerivesFrom,
        0,
        json!({ "sourceMessageId": m2.id, "contextCutMessageId": m2.id }),
    ));
    let dive_turn = turn(&project_id, &dive.id, 0, None, 400);
    let dive_user = msg(
        &project_id,
        &dive.id,
        &dive_turn.id,
        "user",
        if en { "Show me how the context is cut." } else { "具体展示上下文是怎么截断的？" },
        "",
        401,
    );
    let dive_answer = msg(
        &project_id,
        &dive.id,
        &dive_turn.id,
        "assistant",
        if en {
            "This card inherits the root path up to the message you selected. Later root messages are not sent to the model.\n\nOpen **Context** to see the path items, token counts, and any dropped items. The Context snapshot is saved with the run, so you can later explain why the model saw exactly that input."
                .into()
        } else {
            "这张卡片继承根卡片到选中消息为止的路径；根卡片后面新增的消息不会进入模型输入。\n\n打开**上下文**可以看到路径条目、token 数与被丢弃的条目。运行时会保存 Context 快照，之后可以追溯“模型当时到底看到了什么”。"
                .into()
        },
        "",
        402,
    );
    nodes.push(dive_turn.clone());
    edges.push(Edge::new(
        Some(project_id.clone()),
        &dive.id,
        &dive_turn.id,
        EdgeRelation::Contains,
        0,
        json!({}),
    ));
    for (index, message) in [dive_user, dive_answer].into_iter().enumerate() {
        edges.push(Edge::new(
            Some(project_id.clone()),
            &dive_turn.id,
            &message.id,
            EdgeRelation::Contains,
            index as i64,
            json!({}),
        ));
        nodes.push(message);
    }

    // Diverge card, parallel to the root.
    edges.push(Edge::new(
        Some(project_id.clone()),
        &diverge.id,
        &root.id,
        EdgeRelation::ParallelTo,
        0,
        json!({}),
    ));
    let diverge_turn = turn(&project_id, &diverge.id, 0, None, 500);
    let diverge_user = msg(
        &project_id,
        &diverge.id,
        &diverge_turn.id,
        "user",
        if en { "What if I prefer atomic notes?" } else { "如果我更喜欢原子笔记呢？" },
        "",
        501,
    );
    let diverge_answer = msg(
        &project_id,
        &diverge.id,
        &diverge_turn.id,
        "assistant",
        if en {
            "Use **Knowledge** for atomic insights and keep cards for exploration. A knowledge node can cite a message or document chunk, and later runs can include selected knowledge in the compiled context.\n\nThis card is a sibling in the reference UI, but it is still just a node with a `parallel_to` edge."
                .into()
        } else {
            "把原子理解放进**知识**，卡片继续承担探索。知识节点可以引用消息或文档片段，之后的运行可以把选中的知识加入编译上下文。\n\n在表层上它是参考界面里的“发散卡片”，但底层仍然只是一个带 `parallel_to` 边的节点。"
                .into()
        },
        "",
        502,
    );
    nodes.push(diverge_turn.clone());
    edges.push(Edge::new(
        Some(project_id.clone()),
        &diverge.id,
        &diverge_turn.id,
        EdgeRelation::Contains,
        0,
        json!({}),
    ));
    for (index, message) in [diverge_user, diverge_answer].into_iter().enumerate() {
        edges.push(Edge::new(
            Some(project_id.clone()),
            &diverge_turn.id,
            &message.id,
            EdgeRelation::Contains,
            index as i64,
            json!({}),
        ));
        nodes.push(message);
    }

    // Branch card, cut at the second user message.
    edges.push(Edge::new(
        Some(project_id.clone()),
        &branch.id,
        &m1.id,
        EdgeRelation::BranchOf,
        0,
        json!({ "sourceMessageId": m1.id, "contextCutMessageId": m1.id }),
    ));
    let branch_turn = turn(&project_id, &branch.id, 0, None, 600);
    let branch_user = msg(
        &project_id,
        &branch.id,
        &branch_turn.id,
        "user",
        if en { "Can I ignore the tree and just chat?" } else { "我可以忽略卡片树，只线性聊天吗？" },
        "",
        601,
    );
    let branch_answer = msg(
        &project_id,
        &branch.id,
        &branch_turn.id,
        "assistant",
        if en {
            "Yes. Stay on one card and keep sending. The tree is optional navigation, not a prerequisite. If you later want to compare, branch from any earlier message; the old path remains intact."
                .into()
        } else {
            "可以。停在一张卡片里连续发送即可。卡片树是可选导航，不是使用前提。之后想比较时，从任意更早的消息分支；旧路径仍然保留。"
                .into()
        },
        "",
        602,
    );
    nodes.push(branch_turn.clone());
    edges.push(Edge::new(
        Some(project_id.clone()),
        &branch.id,
        &branch_turn.id,
        EdgeRelation::Contains,
        0,
        json!({}),
    ));
    for (index, message) in [branch_user, branch_answer].into_iter().enumerate() {
        edges.push(Edge::new(
            Some(project_id.clone()),
            &branch_turn.id,
            &message.id,
            EdgeRelation::Contains,
            index as i64,
            json!({}),
        ));
        nodes.push(message);
    }

    // A knowledge entry with a source edge.
    let knowledge = Node::new(
        NodeType::Knowledge,
        Some(project_id.clone()),
        if en { "Context is compiled, not replayed" } else { "上下文是编译结果，不是聊天记录" },
        if en {
            "The model does not receive the whole chat. Sporelink compiles a path, optional pinned messages, citations, selected knowledge and document chunks, then applies a token budget. The snapshot is saved with the run."
        } else {
            "模型收到的不是整段聊天。Sporelink 会编译路径消息、可选置顶、引用、选中的知识与文档片段，再按 token 预算裁剪。快照随运行保存。"
        },
        json!({
            "category": "insight",
            "confidence": 0.9,
            "tags": ["context", "architecture"],
            "sourceRefs": [{ "sourceType": "message", "sourceId": m2.id, "title": if en { "Root card · first answer" } else { "根卡片 · 第一条回答" }, "locator": null }],
        }),
    );
    edges.push(Edge::new(
        Some(project_id.clone()),
        &knowledge.id,
        &m2.id,
        EdgeRelation::DerivesFrom,
        0,
        json!({ "sourceMessageId": m2.id }),
    ));
    nodes.push(knowledge);

    store.put_batch(nodes, edges)?;
    Ok(project_id)
}
