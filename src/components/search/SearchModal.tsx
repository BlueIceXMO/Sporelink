import { useEffect, useMemo, useRef, useState } from "react";
import { FileText, Search, Star, X } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import { cn, escapeRegExp, formatDate } from "@/lib/utils";
import { Spinner } from "@/components/common/Primitives";
import { IconButton } from "@/components/common/IconButton";

function Highlighted({ text, query }: { text: string; query: string }) {
  if (!query.trim()) return <>{text}</>;
  const parts = text.split(new RegExp(`(${escapeRegExp(query.trim())})`, "ig"));
  return (
    <>
      {parts.map((part, index) =>
        part.toLowerCase() === query.trim().toLowerCase() ? <mark key={index}>{part}</mark> : <span key={index}>{part}</span>,
      )}
    </>
  );
}

export function SearchModal() {
  const { locale, t } = useI18n();
  const open = useAppStore((s) => s.searchOpen);
  const setOpen = useAppStore((s) => s.setSearchOpen);
  const results = useAppStore((s) => s.searchResults);
  const tookMs = useAppStore((s) => s.searchTookMs);
  const query = useAppStore((s) => s.searchQuery);
  const type = useAppStore((s) => s.searchType);
  const favoriteOnly = useAppStore((s) => s.searchFavoriteOnly);
  const search = useAppStore((s) => s.search);
  const openSearchHit = useAppStore((s) => s.openSearchHit);
  const [value, setValue] = useState(query);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    setValue(query);
    window.setTimeout(() => inputRef.current?.focus(), 40);
  }, [open, query]);

  useEffect(() => {
    if (!open) return;
    const timer = window.setTimeout(async () => {
      setLoading(true);
      await search(value, type, favoriteOnly);
      setLoading(false);
    }, 180);
    return () => window.clearTimeout(timer);
  }, [value, open, type, favoriteOnly, search]);

  const filters = useMemo(
    () => [
      { id: "all", label: t("search.filter.all") },
      { id: "project", label: t("search.filter.projects") },
      { id: "card", label: t("search.filter.cards") },
      { id: "message", label: t("search.filter.messages") },
      { id: "document", label: t("search.filter.documents") },
      { id: "knowledge", label: t("search.filter.knowledge") },
    ],
    [t],
  );

  if (!open) return null;

  return (
    <div className="search-backdrop" onMouseDown={(e) => e.target === e.currentTarget && setOpen(false)}>
      <div className="search-modal" role="dialog" aria-modal="true">
        <div className="search-input-row">
          <Search size={20} />
          <input
            ref={inputRef}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder={t("search.placeholder")}
            onKeyDown={(e) => {
              if (e.key === "Escape") setOpen(false);
              if (e.key === "Enter" && results[0]) {
                e.preventDefault();
                void openSearchHit(results[0]);
              }
            }}
          />
          {loading && <Spinner size={16} />}
          <IconButton label={t("app.close")} size="sm" onClick={() => setOpen(false)}>
            <X size={17} />
          </IconButton>
        </div>
        <div className="search-filters">
          <div className="search-filter-group">
            {filters.map((filter) => (
              <button
                key={filter.id}
                type="button"
                className={cn("search-filter", type === filter.id && "is-active")}
                onClick={() => {
                  void search(value, filter.id, favoriteOnly);
                }}
              >
                {filter.label}
              </button>
            ))}
          </div>
          <button
            type="button"
            className={cn("search-filter", favoriteOnly && "is-active")}
            onClick={() => void search(value, type, !favoriteOnly)}
          >
            <Star size={13} /> {t("search.favoriteOnly")}
          </button>
        </div>
        <div className="search-meta">
          {t("search.results", { count: results.length, ms: tookMs })}
        </div>
        <div className="search-results">
          {results.length === 0 && !loading && (
            <div className="search-empty">
              <h3>{t("search.empty")}</h3>
              <p>{t("search.emptyHint")}</p>
            </div>
          )}
          {results.map((hit) => (
            <button
              key={`${hit.nodeType}:${hit.nodeId}`}
              type="button"
              className="search-hit"
              onClick={() => void openSearchHit(hit)}
            >
              <span className={`search-hit-icon type-${hit.nodeType}`}>
                {hit.nodeType === "document" ? <FileText size={16} /> : <Search size={16} />}
              </span>
              <span className="search-hit-main">
                <span className="search-hit-top">
                  <strong>{hit.title}</strong>
                  <span className="search-hit-project">{hit.projectName}</span>
                </span>
                <span className="search-hit-snippet">
                  <Highlighted text={hit.snippet} query={value} />
                </span>
                <span className="search-hit-meta">
                  {hit.nodeType} · {formatDate(hit.updatedAt, locale)}
                </span>
              </span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
