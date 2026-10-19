# Third-party notices

Sporelink is licensed under GPL-3.0-or-later. This file lists the main third-party components and their licenses. Full license texts for Rust crates are shipped in the Cargo registry used at build time and can be regenerated from `Cargo.lock`.

## Required attribution: VelesDB

**Powered by [VelesDB](https://velesdb.com)**.

- Component: `velesdb-core` 6.x (VelesDB Core)
- License: VelesDB Core License 1.0 (source-available, based on Elastic License 2.0)
- Full text: [`licenses/VelesDB-Core-LICENSE.txt`](licenses/VelesDB-Core-LICENSE.txt)
- Use: embedded, local-first desktop storage inside Sporelink. End users do not receive direct access to VelesDB collections, queries, or administration.
- The attribution link is also visible in Sporelink's Settings → About.

## Main components

| Component | License | Notes |
| --- | --- | --- |
| Tauri 2 (`tauri`, `tauri-build`, plugins) | MIT OR Apache-2.0 | Desktop shell, IPC, dialog/opener plugins |
| React / React DOM | MIT | UI runtime |
| Vite / Rolldown | MIT | Build tooling |
| TypeScript | Apache-2.0 | Types and compiler |
| Zustand | MIT | UI state |
| react-markdown | MIT | Markdown rendering |
| remark-gfm / remark-math | MIT | Markdown extensions |
| rehype-katex / KaTeX | MIT | Math rendering |
| rehype-highlight / highlight.js | MIT / BSD-3-Clause | Code highlighting |
| lucide-react / Lucide | ISC | Icons |
| Manrope | SIL Open Font License 1.1 | UI font |
| JetBrains Mono | SIL Open Font License 1.1 | Code font |
| `reqwest` | MIT OR Apache-2.0 | HTTPS client (rustls) |
| `keyring` | MIT OR Apache-2.0 | OS keychain access |
| `serde` / `serde_json` | MIT OR Apache-2.0 | Serialization |
| `uuid` | Apache-2.0 OR MIT | IDs |
| `tokio` | MIT | Async runtime |
| `lopdf` | MIT | PDF text extraction |
| `zip` | MIT | DOCX/PPTX container parsing |
| `scraper` | MIT | HTML text extraction |
| `sha2` / `hex` | MIT OR Apache-2.0 | Content hashing |
| `tracing` / `tracing-subscriber` | MIT | Local diagnostics |

## Notes

- The `ref/` directory is an Explore reference mirror and is **not** part of this repository or its release artifacts. Explore's copyright and license do not cover Sporelink code.
- No Explore code, assets, fonts, icons, copy, or prompts are copied into Sporelink.
- If you redistribute Sporelink binaries, include this file and `licenses/VelesDB-Core-LICENSE.txt` alongside the GPL-3.0 text.
