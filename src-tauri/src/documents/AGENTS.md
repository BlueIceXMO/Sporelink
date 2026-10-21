# documents/

- 解析在 Rust；前端只拿 chunk 文本。
- PDF 按页、DOCX/PPTX 按段落/幻灯片；保留页码/标题。
- 失败给可行动文案（例如扫描 PDF 需要 OCR，当前不做）。
- 大文件分块，不要一次性塞进上下文。
- 原文留在工作区，不复制进 `.spore`；只存路径、hash、chunk 引用与本地搜索索引。
- 工作区外文件显式导入时复制到 `<workspace>/attachments/`，不保留外部读取权限。
