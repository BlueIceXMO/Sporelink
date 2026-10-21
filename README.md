# Sporelink

Sporelink 是一个本地优先的桌面端知识探索工具。它把对话、文档、知识与引用统一成 **Node / Edge 图**：业务内容是节点，关系是边；表层看到的“根卡片、深潜、发散、分支”由 Rust 依据图拓扑派生，而不是落库的卡片类型。

> 当前版本：`0.1.0-alpha`。这是基础框架版，不是 v0.1 全量完成。范围与状态见 [`docs/feature-parity.md`](docs/feature-parity.md)。

## 技术栈

- Tauri 2 + Rust
- React + TypeScript + Vite
- VelesDB（`velesdb-core`）作为唯一业务存储
- OS keychain 保存模型 API Key
- GPL-3.0-or-later

## 运行

前置：Node.js 20.19+ / 22.12+、Rust stable 1.90+（MSVC）、WebView2、Windows 需要 VS Build Tools 与 Windows SDK。

```bash
cd Sporelink
npm install
npm run tauri dev
```

只构建前端：

```bash
npm run typecheck
npm run build
```

构建安装包：

```bash
npm run tauri build
```

只构建可执行文件（不生成安装包）：

```bash
npm run tauri -- build --debug --no-bundle
```

## 验证

```bash
npm run typecheck
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

端到端冒烟需要一个正在运行、带 WebView2 远程调试端口的应用：

```bash
# 终端 A（应用运行后）
npm run smoke       # 引导 / 示例项目 / Mock 流式 / 上下文
npm run api-smoke   # 图、搜索、文档、知识、摘要、导出、导入、快照
```

`scripts/*.mjs` 通过 `DSH_WEBVIEW_CDP`（默认 `http://127.0.0.1:9222`）连接 WebView2；只用于本地验证，不是产品依赖。

应用配置默认在 `~/.sporelink/`（Windows `C:\Users\<user>\.sporelink\`）。调试时可用环境变量覆盖：

```bash
SPORELINK_CONFIG_DIR=D:\path\to\sporelink-config
SPORELINK_WORKSPACE=D:\path\to\workspace
```

## 数据与安全

- 主数据：`<workspace>/.spore/`，默认工作区是 `Documents\Sporelink`；可自由更换。
- 配置：`~/.sporelink/config.json`，包括当前/最近工作区、主题、语言、布局、模型 profile 与访问模式。
- WebView2 profile：`~/.sporelink/webview2`。
- 启动失败时查看 `~/.sporelink/logs/startup.log`；里面只记路径与错误码，不记消息正文或 Key。
- 原始文档/附件留在工作区，不复制进 `.spore`；工作区外文件显式导入时复制到 `<workspace>/attachments/`。
- `.spore` 与 `.sporelink` 对文件扫描、搜索、AI 文件读取全部隐藏。
- 当前访问模式只有“仅工作区读写”；只读与完全权限是后续规划。
- 业务数据只写入本地 VelesDB；没有账号、云同步、遥测、远程配置或自动更新。
- API Key 只存在 OS keychain；前端只能读取“已设置/未设置”，没有读取 Key 的 IPC。
- 导出不含 Key；导出前会递归剔除 `apiKey`、`authorization`、`secret` 等字段。
- 模型请求由 Rust 直连用户配置的供应商；自定义 Base URL 需要用户确认。

## 目录

```text
docs/            规格、契约、架构、功能对照
src/             React UI
src-tauri/       Tauri 2 + Rust 后端
  src/domain     Node/Edge 模型与 DTO
  src/store      VelesDB 读写、Writer、快照
  src/context    上下文编译器
  src/llm        供应商适配器与流式
  src/documents  文档解析与分块
  src/search     本地全文搜索
  src/export     导入导出与脱敏
  src/commands   Tauri IPC
```

契约优先：改 IPC、存储 schema 或导出 schema 前，先改 [`docs/contracts.md`](docs/contracts.md)。

## 许可证与归属

- 本项目代码：GPL-3.0-or-later，见 [`LICENSE`](LICENSE)。
- **Powered by [VelesDB](https://velesdb.com)**。VelesDB Core 使用 VelesDB Core License 1.0；完整文本随发布物放在 [`licenses/VelesDB-Core-LICENSE.txt`](licenses/VelesDB-Core-LICENSE.txt)。桌面/嵌入式使用被明确允许；不得把 VelesDB 作为托管数据库服务对外提供。
- 其他第三方许可证见 [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)。

## 已知边界

- 不做 OCR、语义向量检索、供应商原生 web search、合并卡、多候选 diff、同步/协作、插件/MCP/CLI。
- 3D 卡片树使用 SVG 数学投影，不是 WebGL。
- 文档支持 PDF/DOCX/PPTX/HTML/Markdown/TXT/JSON/常见代码；复杂排版可能降级为纯文本。
- `ref/` 是 Explore 的只读参考镜像，不属于本仓库，也不进入发布物。
