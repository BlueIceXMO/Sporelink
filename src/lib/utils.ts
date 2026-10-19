import type { CardSurface } from "./types";

export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export function formatDate(value: number, locale: string): string {
  if (!value) return "";
  try {
    return new Intl.DateTimeFormat(locale, {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
    }).format(new Date(value));
  } catch {
    return new Date(value).toLocaleDateString();
  }
}

export function formatDateTime(value: number, locale: string): string {
  if (!value) return "";
  try {
    return new Intl.DateTimeFormat(locale, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(value));
  } catch {
    return new Date(value).toLocaleString();
  }
}

export function relativeTime(value: number, locale: string): string {
  if (!value) return "";
  const diff = Date.now() - value;
  const minute = 60_000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return locale.startsWith("zh") ? "刚刚" : "just now";
  if (diff < hour) {
    const n = Math.floor(diff / minute);
    return locale.startsWith("zh") ? `${n} 分钟前` : `${n} min ago`;
  }
  if (diff < day) {
    const n = Math.floor(diff / hour);
    return locale.startsWith("zh") ? `${n} 小时前` : `${n} h ago`;
  }
  if (diff < 7 * day) {
    const n = Math.floor(diff / day);
    return locale.startsWith("zh") ? `${n} 天前` : `${n} d ago`;
  }
  return formatDate(value, locale);
}

export function surfaceKey(surface: CardSurface): string {
  return `card.${surface}`;
}

export function truncate(text: string, max: number): string {
  if (text.length <= max) return text;
  return text.slice(0, Math.max(0, max - 1)).trimEnd() + "…";
}

export function firstLine(text: string, max = 48): string {
  const line = text.split(/\r?\n/).find((l) => l.trim().length > 0) ?? "";
  return truncate(line.trim(), max);
}

export function estimateTokens(text: string): number {
  let cjk = 0;
  let other = 0;
  for (const ch of text) {
    if (/[\u3400-\u9fff\uf900-\ufaff\u3040-\u30ff\uac00-\ud7af]/.test(ch)) cjk += 1;
    else other += 1;
  }
  return Math.max(1, Math.ceil(cjk + other / 4));
}

export function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

export function formatCost(cost: number, currency = "USD"): string {
  if (!cost) return "—";
  if (cost < 0.01) return `<${currency === "USD" ? "$" : ""}0.01`;
  return `${currency === "USD" ? "$" : ""}${cost.toFixed(2)}`;
}

export function newLocalId(): string {
  return crypto.randomUUID();
}

export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}

export function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName.toLowerCase();
  return (
    tag === "input" ||
    tag === "textarea" ||
    tag === "select" ||
    target.isContentEditable ||
    Boolean(target.closest("[contenteditable='true']"))
  );
}

export function splitTags(text: string): string[] {
  return text
    .split(/[,，\n]/)
    .map((s) => s.trim())
    .filter(Boolean);
}

export function parseHttpUrl(text: string): string | null {
  try {
    const url = new URL(text.trim());
    if (url.protocol !== "http:" && url.protocol !== "https:") return null;
    return url.toString();
  } catch {
    return null;
  }
}

export function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function highlightSnippet(text: string, query: string): string {
  if (!query.trim()) return text;
  const re = new RegExp(escapeRegExp(query.trim()), "ig");
  return text.replace(re, (m) => `<mark>${m}</mark>`);
}
