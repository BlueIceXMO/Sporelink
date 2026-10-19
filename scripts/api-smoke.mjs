import { chromium } from "playwright-core";
import fs from "node:fs";
import path from "node:path";

const cdp = process.env.DSH_WEBVIEW_CDP || "http://127.0.0.1:9222";
const outDir = process.env.SMOKE_OUT || "artifacts";
const dataDir = process.env.SPORELINK_DATA_DIR || "D:/Documents/VibeProjects/Sporelink_ws/.data/sporelink-smoke";

fs.mkdirSync(dataDir, { recursive: true });
fs.mkdirSync(outDir, { recursive: true });
const markdownPath = path.join(dataDir, "smoke-document.md");
fs.writeFileSync(
  markdownPath,
  "# Smoke document\n\nThis document is used to verify local parsing.\n\n## Section two\n\nSporelink stores documents as chunks.\n",
  "utf8",
);

const browser = await chromium.connectOverCDP(cdp);
const page = browser.contexts()[0].pages()[0];
if (!page) throw new Error("no WebView2 page");

const invoke = (command, args = {}) =>
  page.evaluate(
    ([cmd, a]) => window.__TAURI_INTERNALS__.invoke(cmd, a),
    [command, args],
  );

const boot = await invoke("bootstrap");
const projects = await invoke("project_list", { includeArchived: true });
const project = projects[0] || boot.projects?.[0];
if (!project) throw new Error("no project; run smoke.mjs first");
const detail = await invoke("project_get", { id: project.id });
const graph = await invoke("graph_get", { projectId: project.id });
const model = boot.models.find((m) => m.data.provider === "mock") || boot.models[0];
const root = await invoke("card_get", { id: detail.rootCardId });
const context = await invoke("context_preview", {
  input: {
    cardId: detail.rootCardId,
    draftContent: "Verify the compiled context.",
    modelProfileId: model.id,
    context: {
      excludedItemIds: [],
      pinnedMessageIds: [],
      knowledgeIds: [],
      documentChunkIds: [],
    },
  },
});
const search = await invoke("search", { input: { query: "上下文", limit: 20 } });
const doc = await invoke("document_import", { path: markdownPath, projectId: project.id });
const docs = await invoke("document_list", { projectId: project.id });
const knowledge = await invoke("knowledge_list", { projectId: project.id });
const exportPath = path.join(dataDir, "smoke-export.json");
await invoke("vault_export", { path: exportPath });
const snapshot = await invoke("snapshot_create", { label: "API smoke" });
const snapshots = await invoke("snapshot_list");
const trash = await invoke("trash_list");
const test = await invoke("model_test", { id: model.id });
const diagnostics = await invoke("diagnostics");
const sourceMessage = root.turns
  .flatMap((turn) => turn.messages)
  .find((message) => message.role === "assistant");
const dive = await invoke("card_create", {
  input: {
    projectId: project.id,
    parentCardId: root.card.id,
    relation: "dive",
    sourceMessageId: sourceMessage?.id,
    title: "API dive",
  },
});
const summaryKnowledge = await invoke("project_summary", {
  id: project.id,
  modelProfileId: model.id,
});
const checked = await invoke("knowledge_check", {
  input: { id: summaryKnowledge.id, modelProfileId: model.id },
});
const projectExportPath = path.join(dataDir, "smoke-project.json");
await invoke("project_export", {
  id: project.id,
  path: projectExportPath,
  format: "json",
});
const exportedProject = JSON.parse(fs.readFileSync(projectExportPath, "utf8"));
const importedProject = await invoke("project_import", { path: projectExportPath });

const exported = JSON.parse(fs.readFileSync(exportPath, "utf8"));
const exportedText = JSON.stringify(exported);

const summary = {
  project: project.name,
  cards: graph.cards.length,
  edges: graph.edges.length,
  rootSurface: root.card.surfaceKind,
  contextItems: context.items.length,
  contextTokens: context.totalTokens,
  searchHits: search.hits.length,
  documentId: doc.document.id,
  documentChunks: doc.chunks.length,
  documentListCount: docs.length,
  knowledgeCount: knowledge.length,
  exportNodes: exported.nodes.length,
  exportHasSecret: /sk-|apiKey|authorization/i.test(exportedText),
  snapshotId: snapshot.id,
  snapshotCount: snapshots.length,
  trashCount: trash.length,
  modelTestOk: test.ok,
  diagnosticsNodes: diagnostics.counts.nodes,
  diveSurface: dive.card.surfaceKind,
  summaryCategory: summaryKnowledge.category,
  knowledgeFeedback: checked.feedback.slice(0, 80),
  projectExportNodes: exportedProject.nodes.length,
  importedProjectCards: importedProject.project.cardCount,
};

console.log(JSON.stringify(summary, null, 2));

if (graph.cards.length < 4) throw new Error("sample cards missing");
if (root.card.surfaceKind !== "root") throw new Error("root surface wrong");
if (dive.card.surfaceKind !== "dive") throw new Error("dive surface wrong");
if (context.items.length === 0) throw new Error("context empty");
if (search.hits.length === 0) throw new Error("search empty");
if (doc.chunks.length === 0) throw new Error("document chunks empty");
if (exported.nodes.length === 0) throw new Error("export empty");
if (exportedProject.nodes.length === 0) throw new Error("project export empty");
if (importedProject.project.cardCount === 0) throw new Error("project import empty");
if (summary.exportHasSecret) throw new Error("export contains secret-like key");
if (!test.ok) throw new Error("mock model test failed");
if (!checked.feedback) throw new Error("knowledge check empty");

await browser.close();
