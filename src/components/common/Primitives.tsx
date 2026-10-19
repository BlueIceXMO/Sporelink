import React from "react";
import { cn } from "@/lib/utils";
import type { CardSurface } from "@/lib/types";
import { useI18n } from "@/lib/i18n";

const SURFACE_ICON: Record<CardSurface, string> = {
  root: "●",
  dive: "↳",
  diverge: "⇢",
  branch: "⑂",
  child: "○",
  note: "◇",
};

export function SurfaceBadge({ surface, compact = false }: { surface: CardSurface; compact?: boolean }) {
  const { t } = useI18n();
  return (
    <span className={cn("surface-badge", `surface-${surface}`, compact && "is-compact")}>
      <span aria-hidden>{SURFACE_ICON[surface]}</span>
      {!compact && <span>{t(`card.${surface}`)}</span>}
    </span>
  );
}

export function EmptyState({
  icon,
  title,
  body,
  actions,
}: {
  icon?: React.ReactNode;
  title: string;
  body?: string;
  actions?: React.ReactNode;
}) {
  return (
    <div className="empty-state">
      {icon && <div className="empty-icon">{icon}</div>}
      <h3>{title}</h3>
      {body && <p>{body}</p>}
      {actions && <div className="empty-actions">{actions}</div>}
    </div>
  );
}

export function Spinner({ size = 18 }: { size?: number }) {
  return <span className="spinner" style={{ width: size, height: size }} aria-label="loading" />;
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: Array<{ value: T; label: React.ReactNode }>;
  onChange: (value: T) => void;
}) {
  return (
    <div className="segmented">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          className={cn("segmented-item", value === option.value && "is-active")}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export function Switch({
  checked,
  onChange,
  label,
  description,
}: {
  checked: boolean;
  onChange: (value: boolean) => void;
  label: string;
  description?: string;
}) {
  return (
    <label className="switch-row">
      <span className="switch-copy">
        <span className="switch-label">{label}</span>
        {description && <span className="switch-desc">{description}</span>}
      </span>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        className={cn("switch", checked && "is-on")}
        onClick={() => onChange(!checked)}
      >
        <span />
      </button>
    </label>
  );
}

export function Field({
  label,
  hint,
  children,
}: {
  label: React.ReactNode;
  hint?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <label className="field">
      <span className="field-label">{label}</span>
      {children}
      {hint && <span className="field-hint">{hint}</span>}
    </label>
  );
}
