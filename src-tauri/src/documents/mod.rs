use crate::domain::dto::{ChunkDto, ChunkMatch, DocumentDetail, DocumentSummary};
use crate::domain::model::{now_ms, Edge, EdgeRelation, Node, NodeType};
use crate::error::{AppError, AppResult};
use crate::store::Store;
use regex::Regex;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ParsedChunk {
    pub page: Option<u32>,
    pub heading: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct ParsedDoc {
    pub title: String,
    pub file_name: String,
    pub mime: String,
    pub pages: u32,
    pub chunks: Vec<ParsedChunk>,
}

pub fn parse_file(path: &Path) -> AppResult<ParsedDoc> {
    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "document".into());
    let extension = path
        .extension()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let title = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| file_name.clone());
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    match extension.as_str() {
        "pdf" => parse_pdf(path, title, file_name, mime),
        "docx" => parse_docx(path, title, file_name, mime),
        "pptx" => parse_pptx(path, title, file_name, mime),
        "html" | "htm" => parse_html(path, title, file_name, mime),
        "md" | "markdown" | "txt" | "json" | "csv" | "ts" | "tsx" | "js" | "jsx" | "py" | "rs"
        | "toml" | "yaml" | "yml" => {
            let bytes = std::fs::read(path)?;
            let text = String::from_utf8_lossy(&bytes).to_string();
            let chunks = chunk_text(&text, None, None);
            Ok(ParsedDoc {
                title,
                file_name,
                mime,
                pages: 0,
                chunks,
            })
        }
        other => Err(AppError::unsupported(format!(
            "暂不支持 .{other} 文件；支持 PDF、DOCX、PPTX、HTML、Markdown、TXT、JSON 与常见代码文件"
        ))),
    }
}

fn parse_pdf(
    path: &Path,
    title: String,
    file_name: String,
    mime: String,
) -> AppResult<ParsedDoc> {
    let doc = lopdf::Document::load(path)
        .map_err(|e| AppError::parse(format!("PDF 打开失败：{e}")))?;
    let pages = doc.get_pages();
    let page_count = pages.len() as u32;
    let mut chunks = Vec::new();
    for (page_number, _object_id) in pages {
        let text = match doc.extract_text(&[page_number]) {
            Ok(text) => text,
            Err(_) => continue,
        };
        if text.trim().is_empty() {
            continue;
        }
        let mut page_chunks = chunk_text(&text, Some(page_number), None);
        chunks.append(&mut page_chunks);
    }
    if chunks.is_empty() {
        return Err(AppError::parse(
            "PDF 没有提取到可读文本。它可能是扫描件；当前版本不做 OCR。",
        ));
    }
    Ok(ParsedDoc {
        title,
        file_name,
        mime,
        pages: page_count,
        chunks,
    })
}

fn parse_docx(
    path: &Path,
    title: String,
    file_name: String,
    mime: String,
) -> AppResult<ParsedDoc> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::parse(format!("DOCX 打开失败：{e}")))?;
    let mut xml = String::new();
    {
        let mut entry = archive
            .by_name("word/document.xml")
            .map_err(|e| AppError::parse(format!("DOCX 缺少 document.xml：{e}")))?;
        entry.read_to_string(&mut xml)?;
    }
    let text = xml_to_paragraphs(&xml);
    let chunks = chunk_text(&text, None, None);
    Ok(ParsedDoc {
        title,
        file_name,
        mime,
        pages: 0,
        chunks,
    })
}

fn parse_pptx(
    path: &Path,
    title: String,
    file_name: String,
    mime: String,
) -> AppResult<ParsedDoc> {
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::parse(format!("PPTX 打开失败：{e}")))?;
    let mut slide_names = (0..archive.len())
        .filter_map(|index| {
            archive
                .by_index(index)
                .ok()
                .map(|entry| entry.name().to_string())
        })
        .filter(|name| name.starts_with("ppt/slides/slide") && name.ends_with(".xml"))
        .collect::<Vec<_>>();
    slide_names.sort_by_key(|name| {
        name.trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml")
            .parse::<u32>()
            .unwrap_or(0)
    });
    let text_re = Regex::new(r"<a:t[^>]*>(.*?)</a:t>").expect("valid regex");
    let mut chunks = Vec::new();
    for (index, name) in slide_names.iter().enumerate() {
        let mut xml = String::new();
        if let Ok(mut entry) = archive.by_name(name) {
            let _ = entry.read_to_string(&mut xml);
        }
        let text = text_re
            .captures_iter(&xml)
            .map(|cap| unescape_xml(&cap[1]))
            .collect::<Vec<_>>()
            .join(" ");
        if text.trim().is_empty() {
            continue;
        }
        chunks.push(ParsedChunk {
            page: Some(index as u32 + 1),
            heading: Some(format!("Slide {}", index + 1)),
            text: text.trim().to_string(),
        });
    }
    if chunks.is_empty() {
        return Err(AppError::parse("PPTX 没有提取到可读文本"));
    }
    let pages = chunks.len() as u32;
    Ok(ParsedDoc {
        title,
        file_name,
        mime,
        pages,
        chunks,
    })
}

fn parse_html(
    path: &Path,
    title: String,
    file_name: String,
    mime: String,
) -> AppResult<ParsedDoc> {
    let html = std::fs::read_to_string(path)?;
    let document = scraper::Html::parse_document(&html);
    let title_selector = scraper::Selector::parse("title").ok();
    let body_selector = scraper::Selector::parse("body").ok();
    let page_title = title_selector
        .and_then(|selector| document.select(&selector).next())
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(title);
    let text = body_selector
        .and_then(|selector| document.select(&selector).next())
        .map(|el| el.text().collect::<Vec<_>>().join("\n"))
        .unwrap_or_else(|| document.root_element().text().collect::<Vec<_>>().join("\n"));
    let chunks = chunk_text(&text, None, None);
    Ok(ParsedDoc {
        title: page_title,
        file_name,
        mime,
        pages: 0,
        chunks,
    })
}

fn xml_to_paragraphs(xml: &str) -> String {
    let with_breaks = xml
        .replace("</w:p>", "\n")
        .replace("</w:tr>", "\n")
        .replace("</w:tc>", " ");
    let tag_re = Regex::new(r"<[^>]+>").expect("valid regex");
    let plain = tag_re.replace_all(&with_breaks, "");
    unescape_xml(&plain)
}

fn unescape_xml(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn chunk_text(text: &str, page: Option<u32>, heading: Option<String>) -> Vec<ParsedChunk> {
    let paragraphs = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_heading = heading.clone();
    for paragraph in paragraphs {
        let is_heading = paragraph.starts_with('#') || (paragraph.len() < 80 && !paragraph.ends_with('.'));
        if is_heading && paragraph.starts_with('#') {
            current_heading = Some(paragraph.trim_start_matches('#').trim().to_string());
        }
        if !current.is_empty() && current.len() + paragraph.len() > 1400 {
            chunks.push(ParsedChunk {
                page,
                heading: current_heading.clone(),
                text: current.trim().to_string(),
            });
            current.clear();
        }
        current.push_str(paragraph);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        chunks.push(ParsedChunk {
            page,
            heading: current_heading,
            text: current.trim().to_string(),
        });
    }
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(ParsedChunk {
            page,
            heading,
            text: text.trim().to_string(),
        });
    }
    chunks
}

pub fn import_document(
    store: &Store,
    path: &Path,
    project_id: Option<String>,
) -> AppResult<DocumentDetail> {
    let parsed = parse_file(path)?;
    let bytes = std::fs::read(path)?;
    let hash = hex::encode(Sha256::digest(&bytes));
    let document = Node::new(
        NodeType::Document,
        project_id.clone(),
        parsed.title.clone(),
        "",
        json!({
            "fileName": parsed.file_name,
            "mime": parsed.mime,
            "pages": parsed.pages,
            "hash": hash,
            "chunkCount": parsed.chunks.len(),
        }),
    );
    let mut nodes = vec![document.clone()];
    let mut edges = Vec::new();
    for (index, chunk) in parsed.chunks.iter().enumerate() {
        let node = Node::new(
            NodeType::Chunk,
            project_id.clone(),
            chunk.heading.clone().unwrap_or_else(|| format!("Chunk {}", index + 1)),
            &chunk.text,
            json!({
                "documentId": document.id,
                "index": index,
                "page": chunk.page,
                "heading": chunk.heading,
            }),
        );
        edges.push(Edge::new(
            project_id.clone(),
            &document.id,
            &node.id,
            EdgeRelation::Contains,
            index as i64,
            json!({}),
        ));
        nodes.push(node);
    }
    store.put_batch(nodes, edges)?;
    document_detail(store, &document.id)
}

pub async fn import_url(
    client: &reqwest::Client,
    store: &Store,
    url: &str,
    project_id: Option<String>,
) -> AppResult<DocumentDetail> {
    let parsed_url = url::Url::parse(url).map_err(|e| AppError::invalid(format!("URL 无效：{e}")))?;
    if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
        return Err(AppError::invalid("只支持 http/https URL"));
    }
    let response = client
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| AppError::new("provider_unavailable", format!("抓取失败：{e}")))?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::new(
            "provider_unavailable",
            format!("抓取失败：HTTP {status}"),
        ));
    }
    let html = response.text().await.map_err(|e| AppError::new("provider_unavailable", e.to_string()))?;
    let document = scraper::Html::parse_document(&html);
    let title_selector = scraper::Selector::parse("title").ok();
    let title = title_selector
        .and_then(|selector| document.select(&selector).next())
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| parsed_url.host_str().unwrap_or("Web page").to_string());
    let body_selector = scraper::Selector::parse("body").ok();
    let text = body_selector
        .and_then(|selector| document.select(&selector).next())
        .map(|el| el.text().collect::<Vec<_>>().join("\n"))
        .unwrap_or_else(|| document.root_element().text().collect::<Vec<_>>().join("\n"));
    let chunks = chunk_text(&text, None, None);
    let hash = hex::encode(Sha256::digest(html.as_bytes()));
    let doc_node = Node::new(
        NodeType::Document,
        project_id.clone(),
        title,
        "",
        json!({
            "fileName": parsed_url.as_str(),
            "mime": "text/html",
            "pages": 0,
            "hash": hash,
            "chunkCount": chunks.len(),
        }),
    );
    let mut nodes = vec![doc_node.clone()];
    let mut edges = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let node = Node::new(
            NodeType::Chunk,
            project_id.clone(),
            chunk.heading.clone().unwrap_or_else(|| format!("Chunk {}", index + 1)),
            &chunk.text,
            json!({
                "documentId": doc_node.id,
                "index": index,
                "page": chunk.page,
                "heading": chunk.heading,
            }),
        );
        edges.push(Edge::new(
            project_id.clone(),
            &doc_node.id,
            &node.id,
            EdgeRelation::Contains,
            index as i64,
            json!({}),
        ));
        nodes.push(node);
    }
    store.put_batch(nodes, edges)?;
    document_detail(store, &doc_node.id)
}

pub fn document_summary(node: &Node, chunks: usize) -> DocumentSummary {
    DocumentSummary {
        id: node.id.clone(),
        project_id: node.project_id.clone(),
        title: node.title.clone(),
        file_name: node.data_str("fileName").unwrap_or("").to_string(),
        mime: node.data_str("mime").unwrap_or("").to_string(),
        pages: node.data_i64("pages").unwrap_or(0) as u32,
        chunk_count: chunks,
        created_at: node.created_at,
        updated_at: node.updated_at,
    }
}

pub fn document_detail(store: &Store, id: &str) -> AppResult<DocumentDetail> {
    let document = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Document)
        .ok_or_else(|| AppError::not_found("document"))?;
    let mut chunks = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Chunk
                && node.is_live()
                && node.data_str("documentId") == Some(id)
        })
        .cloned()
        .collect::<Vec<_>>();
    chunks.sort_by_key(|chunk| chunk.data_i64("index").unwrap_or(0));
    let chunk_dtos = chunks
        .iter()
        .map(|chunk| ChunkDto {
            id: chunk.id.clone(),
            index: chunk.data_i64("index").unwrap_or(0) as usize,
            page: chunk.data_i64("page").map(|p| p as u32),
            heading: chunk.data_str("heading").map(str::to_string),
            text: chunk.body.clone(),
        })
        .collect::<Vec<_>>();
    let summary = document_summary(&document, chunk_dtos.len());
    Ok(DocumentDetail {
        document: summary,
        chunks: chunk_dtos,
    })
}

pub fn list_documents(store: &Store, project_id: Option<&str>) -> Vec<DocumentSummary> {
    let mut docs = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Document
                && node.is_live()
                && project_id.map(|p| node.project_id.as_deref() == Some(p)).unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    docs.sort_by_key(|doc| -doc.updated_at);
    docs.iter()
        .map(|doc| {
            let chunks = store
                .state()
                .read()
                .nodes
                .values()
                .filter(|node| {
                    node.node_type == NodeType::Chunk
                        && node.is_live()
                        && node.data_str("documentId") == Some(&doc.id)
                })
                .count();
            document_summary(doc, chunks)
        })
        .collect()
}

pub fn delete_document(store: &Store, id: &str) -> AppResult<()> {
    let document = store
        .live_node(id)
        .filter(|node| node.node_type == NodeType::Document)
        .ok_or_else(|| AppError::not_found("document"))?;
    let mut document = document;
    document.deleted_at = Some(now_ms());
    let mut nodes = vec![document];
    for chunk in store.state().read().nodes.values() {
        if chunk.node_type == NodeType::Chunk
            && chunk.is_live()
            && chunk.data_str("documentId") == Some(id)
        {
            let mut chunk = chunk.clone();
            chunk.deleted_at = Some(now_ms());
            nodes.push(chunk);
        }
    }
    store.put_batch(nodes, vec![])?;
    Ok(())
}

pub fn search_document(store: &Store, id: &str, query: &str) -> Vec<ChunkMatch> {
    let q = query.to_lowercase();
    let mut matches = store
        .state()
        .read()
        .nodes
        .values()
        .filter(|node| {
            node.node_type == NodeType::Chunk
                && node.is_live()
                && node.data_str("documentId") == Some(id)
                && node.body.to_lowercase().contains(&q)
        })
        .map(|node| {
            let lower = node.body.to_lowercase();
            let byte_index = lower.find(&q).unwrap_or(0).min(node.body.len());
            let char_index = node.body[..byte_index].chars().count();
            let chars = node.body.chars().collect::<Vec<_>>();
            let start = char_index.saturating_sub(60);
            let end = (char_index + query.chars().count() + 90).min(chars.len());
            let mut snippet = chars[start..end].iter().collect::<String>().replace('\n', " ");
            if start > 0 {
                snippet = format!("…{snippet}");
            }
            if end < chars.len() {
                snippet.push('…');
            }
            ChunkMatch {
                chunk_id: node.id.clone(),
                page: node.data_i64("page").map(|p| p as u32),
                snippet,
                score: 1.0,
            }
        })
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_long_text() {
        let text = "Paragraph one.\n\nParagraph two.\n\n".repeat(200);
        let chunks = chunk_text(&text, Some(1), None);
        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|chunk| chunk.text.len() <= 1400));
        assert_eq!(chunks[0].page, Some(1));
    }
}
