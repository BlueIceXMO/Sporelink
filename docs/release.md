# 发布、安装包与代码签名

> 状态：计划中。v0.1.0-alpha 当前以 portable exe 为开发/验收产物；安装包可以生成，但尚未签名。

## 1. 开发阶段策略

- 日常开发、调试、自动冒烟使用 portable exe 或 `npm run tauri dev`；不把安装/卸载作为迭代步骤。
- 安装包只在里程碑、beta 或正式发布时生成。
- 产物统一从 `src-tauri/tauri.conf.json` 的版本、图标、publisher、resources 生成。
- 不把未签名安装包当作“正式发布”；未签名 exe 和安装包都会触发 Windows SmartScreen“未知发布者”。

## 2. 安装包

Tauri 2 / NSIS：

```bash
npm run tauri build
```

产物：

```text
src-tauri/target/.../bundle/nsis/Sporelink_<version>_x64-setup.exe
```

约定：

- 安装模式：currentUser，安装到 `%LOCALAPPDATA%\Sporelink`。
- 包含开始菜单、桌面快捷方式、卸载器、WebView2 bootstrapper。
- 安装包、portable exe 都要附带 `LICENSE`、`THIRD_PARTY_NOTICES.md`、VelesDB 许可证。
- 安装包和 portable 共用 `~/.sporelink` 配置与工作区 `.spore` 数据；卸载不删除用户数据。

## 3. 代码签名计划

### 3.1 首选：SignPath Foundation

- 面向符合条件的开源项目提供免费代码签名；私钥存放在 SignPath 的 HSM。
- 通过 SignPath.io 与 GitHub Actions / CI 集成，不把私钥放本地。
- 由 SignPath 验证“二进制由公开仓库构建”，发布者通常显示为 SignPath Foundation，而不是个人 GitHub 名。
- 申请前提：公开仓库、OSI 认可许可证（本项目 GPL-3.0-or-later）、可复现的 CI 构建、项目维护者授权。
- 参考：<https://signpath.org/>、<https://signpath.org/apply>

### 3.2 备选：自有证书

| 方案 | 适用 | 说明 |
| --- | --- | --- |
| Certum Open Source | GitHub 项目达到其开源免费门槛 | 当前门槛为 GitHub Star > 1k、至少一个 Release、申请人中国国籍；否则付费云签名 |
| Certum / SSL.com / Sectigo / DigiCert OV | 需要显示自有发布者名 | 价格较低，SmartScreen 信誉需积累 |
| EV 代码签名 | 尽快建立 SmartScreen 信誉 | 价格更高，私钥在硬件/云 HSM，CI 需云签名 |
| Azure Artifact Signing | 有 Azure/组织身份 | 微软托管证书、HSM、时间戳；按订阅/签名量计费 |

如果发布者要显示 `BlueIceXMO`，证书主题必须是 CA 验证过的个人/组织身份；GitHub 用户名本身不是 CA 身份证明。

### 3.3 签名顺序

1. 在 CI 中构建 release `sporelink.exe`。
2. 签名 `sporelink.exe`（SHA-256 + RFC 3161 时间戳）。
3. 生成 NSIS 安装包。
4. 签名安装包。
5. 验证 `Get-AuthenticodeSignature` 为 `Valid`，并检查时间戳。
6. 发布 SHA-256 校验和；可附加 Sigstore/GPG 完整性签名。
7. 不使用自签名证书作为公开分发方案。

Tauri 集成点：

- `bundle.windows.signCommand` / 证书指纹 / 云签名 CLI（以实际证书方案为准）。
- SignPath 使用其 GitHub Action；Azure Artifact Signing 使用对应 signing integration。
- 任何签名私钥、PFX、Token PIN 都不得进入仓库、日志、前端或导出。

## 4. 启动性能与 SmartScreen

- SmartScreen 的“未知发布者”是 Authenticode 信任问题；安装版和 portable 都受影响，签名后才能消除。
- 点“仍要运行”后的等待可能来自：SmartScreen/Defender 扫描未签名 exe、首次 WebView2 profile 初始化、应用同步打开 VelesDB/探测工作区路径。
- 性能目标：冷启动 < 2s（不含模型加载）。若长期超过，先看 `~/.sporelink/logs/startup.log` 的时间戳，再决定是否把工作区初始化移出 Tauri `setup` 同步路径。
- 首次 WebView2 初始化允许较慢；第二次启动应明显加快。

## 5. 发布检查清单

- [ ] `npm run typecheck`、`npm run build`、`cargo test` 通过
- [ ] UI/API 冒烟通过（Mock）
- [ ] 版本号、publisher、copyright、图标一致
- [ ] 安装包与 portable 都包含完整许可证
- [ ] `sporelink.exe` 与安装包均已签名且时间戳有效
- [ ] 发布 SHA-256；GitHub Release 说明数据目录与签名状态
- [ ] 卸载不删除 `~/.sporelink` 与工作区
- [ ] 至少在一台干净 Windows 机器上验证安装、启动、卸载
