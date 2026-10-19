import { useEffect, useMemo, useRef, useState } from "react";
import { Maximize2, RotateCcw } from "lucide-react";
import { useAppStore } from "@/state/store";
import { useI18n } from "@/lib/i18n";
import type { CardSurface, CardView, GraphView } from "@/lib/types";
import { cn } from "@/lib/utils";
import { IconButton } from "@/components/common/IconButton";
import { EmptyState } from "@/components/common/Primitives";

type Point3 = { x: number; y: number; z: number };

const SURFACE_COLOR: Record<CardSurface, string> = {
  root: "var(--brand)",
  dive: "var(--brand)",
  diverge: "#8db3ff",
  branch: "#ffad7a",
  child: "var(--text-tertiary)",
  note: "#c0a0ff",
};

function treeLayout(graph: GraphView): Map<string, Point3> {
  const cards = new Map(graph.cards.map((c) => [c.id, c]));
  const children = new Map<string, string[]>();
  for (const edge of graph.edges) {
    if (edge.relation === "contains" && cards.has(edge.from) && cards.has(edge.to)) {
      const list = children.get(edge.from) ?? [];
      list.push(edge.to);
      children.set(edge.from, list);
    }
  }
  const positions = new Map<string, Point3>();
  const root =
    graph.cards.find((c) => c.id === graph.rootCardId) ??
    graph.cards.find((c) => !c.parentId) ??
    graph.cards[0];
  if (!root) return positions;

  const visited = new Set<string>();
  const walk = (id: string, depth: number, index: number, count: number) => {
    if (visited.has(id)) return;
    visited.add(id);
    const card = cards.get(id);
    if (!card) return;
    if (card.position) {
      positions.set(id, { x: card.position[0], y: card.position[1], z: card.position[2] });
    } else {
      const spread = Math.max(1, count);
      const angle = (index / spread) * Math.PI * 2 - Math.PI / 2;
      const radius = depth === 0 ? 0 : 230 + depth * 100;
      const x = depth === 0 ? 0 : Math.cos(angle) * radius;
      const y = depth * 120 + (depth === 0 ? 0 : Math.sin(angle) * radius * 0.62);
      const z = depth === 0 ? 0 : Math.sin(angle) * radius;
      positions.set(id, { x, y, z });
    }
    const kids = children.get(id) ?? [];
    kids.forEach((kid, i) => walk(kid, depth + 1, i, kids.length));
  };
  walk(root.id, 0, 0, 1);
  // Any orphaned cards (shouldn't happen) get a small ring.
  graph.cards.forEach((card, i) => {
    if (!positions.has(card.id)) {
      positions.set(card.id, {
        x: 180 + (i % 3) * 120,
        y: 80 + Math.floor(i / 3) * 90,
        z: -120,
      });
    }
  });
  return positions;
}

function rotate(point: Point3, yaw: number, pitch: number): Point3 {
  const cy = Math.cos(yaw);
  const sy = Math.sin(yaw);
  const x1 = point.x * cy - point.z * sy;
  const z1 = point.x * sy + point.z * cy;
  const cp = Math.cos(pitch);
  const sp = Math.sin(pitch);
  const y2 = point.y * cp - z1 * sp;
  const z2 = point.y * sp + z1 * cp;
  return { x: x1, y: y2, z: z2 };
}

export function CardTree3D() {
  const { t } = useI18n();
  const graph = useAppStore((s) => s.graph);
  const currentCard = useAppStore((s) => s.currentCard);
  const openCard = useAppStore((s) => s.openCard);
  const tree2d = useAppStore((s) => s.tree2d);
  const setTree2d = useAppStore((s) => s.setTree2d);
  const reducedMotion = useAppStore((s) => s.settings?.layout.reducedMotion ?? false);

  const wrapRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 320, h: 600 });
  const [yaw, setYaw] = useState(-0.45);
  const [pitch, setPitch] = useState(0.18);
  const [zoom, setZoom] = useState(1);
  const drag = useRef<{ x: number; y: number; yaw: number; pitch: number; moved: boolean } | null>(null);

  useEffect(() => {
    const el = wrapRef.current;
    if (!el) return;
    const observer = new ResizeObserver(() => {
      setSize({ w: el.clientWidth, h: el.clientHeight });
    });
    observer.observe(el);
    setSize({ w: el.clientWidth, h: el.clientHeight });
    return () => observer.disconnect();
  }, []);

  const layout = useMemo(() => (graph ? treeLayout(graph) : new Map<string, Point3>()), [graph]);
  const cards = graph?.cards ?? [];
  const isFlat = tree2d || reducedMotion;

  const projected = useMemo(() => {
    const items = cards.map((card) => {
      const raw = layout.get(card.id) ?? { x: 0, y: 0, z: 0 };
      const rotated = isFlat ? { ...raw, z: 0 } : rotate(raw, yaw, pitch);
      return { card, rawX: rotated.x, rawY: rotated.y, depth: rotated.z };
    });
    const xs = items.map((item) => item.rawX);
    const ys = items.map((item) => item.rawY);
    const minX = Math.min(...xs, 0);
    const maxX = Math.max(...xs, 0);
    const minY = Math.min(...ys, 0);
    const maxY = Math.max(...ys, 0);
    const spanX = Math.max(1, maxX - minX);
    const spanY = Math.max(1, maxY - minY);
    const fit = Math.min((size.w - 110) / spanX, (size.h - 150) / spanY, 1.15);
    const baseScale = (isFlat ? 0.88 : 1.0) * fit * zoom;
    const midX = (minX + maxX) / 2;
    const midY = (minY + maxY) / 2;
    const focal = 760;
    const positioned = items.map((item) => {
      const persp = focal / (focal + item.depth + 420);
      const radiusScale = Math.max(0.7, Math.min(1.2, persp));
      return {
        card: item.card,
        x: size.w / 2 + (item.rawX - midX) * baseScale,
        y: size.h / 2 - 18 + (item.rawY - midY) * baseScale,
        depth: item.depth,
        scale: baseScale * radiusScale,
        radius: (item.card.id === currentCard?.card.id ? 14 : 9) * radiusScale,
      };
    });
    return positioned.sort((a, b) => b.depth - a.depth);
  }, [cards, layout, yaw, pitch, zoom, size, currentCard?.card.id, isFlat]);

  const nodeById = useMemo(() => new Map(projected.map((p) => [p.card.id, p])), [projected]);

  if (!graph || cards.length === 0) {
    return (
      <aside className="tree-panel">
        <div className="tree-head">
          <span>{t("tree.title")}</span>
        </div>
        <EmptyState title={t("tree.empty")} />
      </aside>
    );
  }

  return (
    <aside className={cn("tree-panel", isFlat && "is-flat")}>
      <div className="tree-head">
        <div>
          <span className="tree-title">{t("tree.title")}</span>
          <span className="tree-count">{cards.length}</span>
        </div>
        <div className="tree-tools">
          <IconButton
            label={isFlat ? t("tree.threeD") : t("tree.twoD")}
            size="sm"
            onClick={() => void setTree2d(!tree2d)}
          >
            <Maximize2 size={14} />
          </IconButton>
          <IconButton
            label={t("app.reset")}
            size="sm"
            onClick={() => {
              setYaw(-0.45);
              setPitch(0.18);
              setZoom(1);
            }}
          >
            <RotateCcw size={14} />
          </IconButton>
        </div>
      </div>
      <div
        ref={wrapRef}
        className="tree-canvas"
        onPointerDown={(e) => {
          drag.current = { x: e.clientX, y: e.clientY, yaw, pitch, moved: false };
          (e.target as Element).setPointerCapture?.(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (!drag.current) return;
          const dx = e.clientX - drag.current.x;
          const dy = e.clientY - drag.current.y;
          if (Math.abs(dx) + Math.abs(dy) > 3) drag.current.moved = true;
          if (!isFlat) {
            setYaw(drag.current.yaw + dx * 0.008);
            setPitch(Math.max(-1.1, Math.min(1.1, drag.current.pitch + dy * 0.006)));
          }
        }}
        onPointerUp={() => {
          drag.current = null;
        }}
        onPointerLeave={() => {
          drag.current = null;
        }}
        onWheel={(e) => {
          e.preventDefault();
          setZoom((z) => Math.max(0.45, Math.min(2.4, z - e.deltaY * 0.0012)));
        }}
      >
        <svg width={size.w} height={size.h} role="img" aria-label={t("tree.title")}>
          <defs>
            <filter id="node-glow" x="-80%" y="-80%" width="260%" height="260%">
              <feGaussianBlur stdDeviation="5" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <g className="tree-edges">
            {graph.edges
              .filter((e) => e.relation === "contains" || e.relation === "parallel_to")
              .map((edge) => {
                const from = nodeById.get(edge.from);
                const to = nodeById.get(edge.to);
                if (!from || !to) return null;
                const isParallel = edge.relation === "parallel_to";
                return (
                  <line
                    key={edge.id}
                    x1={from.x}
                    y1={from.y}
                    x2={to.x}
                    y2={to.y}
                    className={cn("tree-edge", isParallel && "parallel")}
                    strokeWidth={isParallel ? 1 : Math.max(0.8, 1.6 * to.scale)}
                  />
                );
              })}
          </g>
          <g className="tree-nodes">
            {projected.map(({ card, x, y, radius }) => {
              const active = card.id === currentCard?.card.id;
              const showLabel = cards.length <= 90 || active || radius > 10;
              return (
                <g
                  key={card.id}
                  className={cn("tree-node", active && "is-current")}
                  transform={`translate(${x}, ${y})`}
                  onClick={(e) => {
                    e.stopPropagation();
                    if (!drag.current?.moved) void openCard(card.id);
                  }}
                  onPointerDown={(e) => e.stopPropagation()}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" || e.key === " ") void openCard(card.id);
                  }}
                >
                  <circle
                    r={radius + 7}
                    className="tree-node-halo"
                    style={{ stroke: SURFACE_COLOR[card.surfaceKind] }}
                  />
                  <circle
                    r={radius}
                    className="tree-node-core"
                    style={{ fill: SURFACE_COLOR[card.surfaceKind] }}
                    filter={active ? "url(#node-glow)" : undefined}
                  />
                  {showLabel && (
                    <text
                      y={radius + 17}
                      className="tree-node-label"
                      textAnchor="middle"
                    >
                      {card.title.length > 16 ? `${card.title.slice(0, 15)}…` : card.title}
                    </text>
                  )}
                </g>
              );
            })}
          </g>
        </svg>
      </div>
      <div className="tree-legend">
        <span style={{ color: SURFACE_COLOR.root }}>{t("card.root")}</span>
        <span style={{ color: SURFACE_COLOR.dive }}>{t("card.dive")}</span>
        <span style={{ color: SURFACE_COLOR.diverge }}>{t("card.diverge")}</span>
        <span style={{ color: SURFACE_COLOR.branch }}>{t("card.branch")}</span>
      </div>
      <div className="tree-hint">{t("tree.hint")}</div>
    </aside>
  );
}
