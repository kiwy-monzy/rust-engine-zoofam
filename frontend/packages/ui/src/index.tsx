/** @gateway/ui — small, dependency-free Tailwind primitives. */

import { useEffect } from "react";
import type { ButtonHTMLAttributes, HTMLAttributes, InputHTMLAttributes, ReactNode, TextareaHTMLAttributes } from "react";

export function cn(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

// ---------------------------------------------------------------- button --

type Variant = "primary" | "ghost" | "danger";

const VARIANTS: Record<Variant, string> = {
  primary:
    "bg-[var(--accent)] text-white hover:bg-[var(--accent-strong)] focus-visible:outline-[var(--accent)] disabled:bg-[var(--accent)]/40",
  ghost:
    "bg-[var(--app-card-2)] text-[var(--app-fg)] hover:bg-[var(--sb-hover)] focus-visible:outline-zinc-400 border border-[var(--app-border)]",
  danger:
    "bg-red-500/10 text-red-700 hover:bg-red-500/20 dark:text-red-300 focus-visible:outline-red-500 border border-red-500/30",
};

export function Button({
  variant = "primary",
  loading = false,
  className,
  children,
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant; loading?: boolean }) {
  return (
    <button
      {...rest}
      disabled={rest.disabled || loading}
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-lg px-4 py-2 text-sm font-medium",
        "transition-colors focus-visible:outline-2 focus-visible:outline-offset-2",
        "disabled:cursor-not-allowed",
        VARIANTS[variant],
        className,
      )}
    >
      {loading && <Spinner className="h-3.5 w-3.5" />}
      {children}
    </button>
  );
}

// ----------------------------------------------------------------- input --

export function Input({ className, ...rest }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...rest}
      className={cn(
        "w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]",
        "placeholder:text-[var(--app-fg-faint)] focus:border-[var(--accent)] focus:outline-none",
        className,
      )}
    />
  );
}

export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block space-y-1.5">
      <span className="text-xs font-medium uppercase tracking-wider text-[var(--app-fg-muted)]">{label}</span>
      {children}
    </label>
  );
}

export function Textarea({
  className,
  ...rest
}: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      {...rest}
      className={cn(
        "w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]",
        "placeholder:text-[var(--app-fg-faint)] focus:border-[var(--accent)] focus:outline-none",
        "font-mono text-xs leading-relaxed",
        className,
      )}
    />
  );
}

// ------------------------------------------------------------------ card --

export function Card({ className, ...rest }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...rest}
      className={cn(
        "rounded-xl border border-[var(--app-border)] bg-[var(--app-card)] shadow-sm shadow-black/10",
        className,
      )}
    />
  );
}

// -------------------------------------------------------------- feedback --

export function Spinner({ className }: { className?: string }) {
  return (
    <span
      aria-label="loading"
      className={cn(
        "inline-block h-5 w-5 animate-spin rounded-full border-2 border-[var(--app-border)] border-t-[var(--accent)]",
        className,
      )}
    />
  );
}

export function Alert({
  tone = "error",
  children,
}: {
  tone?: "error" | "info" | "success";
  children: ReactNode;
}) {
  const tones = {
    error: "border-red-500/30 bg-red-500/10 text-red-700 dark:text-red-200",
    info: "border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-200",
    success: "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-200",
  } as const;
  return (
    <div role="alert" className={cn("rounded-lg border px-3 py-2 text-sm", tones[tone])}>
      {children}
    </div>
  );
}

export function PageHeader({
  title,
  description,
  actions,
}: {
  title: string;
  description?: string;
  actions?: ReactNode;
}) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-3">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
        {description && <p className="mt-0.5 max-w-3xl text-sm text-[var(--app-fg-muted)]">{description}</p>}
      </div>
      {actions}
    </header>
  );
}

export function EmptyState({ message }: { message: string }) {
  return (
    <div className="rounded-lg border border-dashed border-[var(--app-border)] px-6 py-10 text-center text-sm text-[var(--app-fg-muted)]">
      {message}
    </div>
  );
}

// ----------------------------------------------------------------- table --

export function Table({ head, children }: { head: string[]; children: ReactNode }) {
  return (
    <div className="overflow-x-auto rounded-xl border border-[var(--app-border)]">
      <table className="w-full text-left text-sm">
        <thead>
          <tr className="border-b border-[var(--app-border)] bg-[var(--app-card-2)]">
            {head.map((h) => (
              <th key={h} className="px-4 py-2.5 text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-[var(--app-border)]">{children}</tbody>
      </table>
    </div>
  );
}

export function Td({ className, children }: { className?: string; children: ReactNode }) {
  return <td className={cn("px-4 py-2.5 align-middle", className)}>{children}</td>;
}

export function Badge({
  tone = "zinc",
  children,
}: {
  tone?: "zinc" | "green" | "red";
  children: ReactNode;
}) {
  const tones = {
    zinc: "bg-[var(--app-card-2)] text-[var(--app-fg-muted)] border border-[var(--app-border)]",
    green: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
    red: "bg-red-500/15 text-red-700 dark:text-red-300",
  } as const;
  return (
    <span className={cn("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium", tones[tone])}>
      {children}
    </span>
  );
}

// ------------------------------------------------------------- side panel --

export function SidePanel({
  open,
  onClose,
  title,
  subtitle,
  children,
  footer,
  width,
}: {
  open: boolean;
  onClose: () => void;
  title: ReactNode;
  subtitle?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  width?: number;
}) {
  useEffect(() => {
    if (!open) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  const w = width ?? 360;
  return (
    <>
      <div
        aria-hidden={!open}
        onClick={onClose}
        className={cn(
          "fixed inset-0 z-[110] bg-black/40 backdrop-blur-[1px] transition-opacity duration-200",
          open ? "opacity-100" : "pointer-events-none opacity-0",
        )}
      />
      <aside
        role="dialog"
        aria-modal="true"
        aria-hidden={!open}
        style={{ width: `${w}px` }}
        className={cn(
          "fixed right-0 top-0 z-[120] flex h-dvh max-w-[94vw] flex-col",
          "border-l border-[var(--app-border)] bg-[var(--sb-bg)] shadow-2xl shadow-black/30",
          "transition-transform duration-200 ease-out",
          open ? "translate-x-0" : "translate-x-full",
        )}
      >
        <header className="flex items-start justify-between gap-3 border-b border-[var(--app-border)] px-5 py-4">
          <div className="min-w-0">
            <h2 className="truncate text-sm font-semibold tracking-tight">{title}</h2>
            {subtitle && (
              <p className="mt-0.5 truncate text-xs text-[var(--app-fg-muted)]">{subtitle}</p>
            )}
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close panel"
            className="grid h-7 w-7 shrink-0 place-items-center rounded-lg text-[var(--app-fg-muted)] transition-colors hover:bg-[var(--sb-hover)] hover:text-[var(--app-fg)]"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <line x1="6" y1="6" x2="18" y2="18" />
              <line x1="18" y1="6" x2="6" y2="18" />
            </svg>
          </button>
        </header>
        <div className="flex-1 space-y-6 overflow-y-auto px-5 py-5">{children}</div>
        {footer && (
          <div className="border-t border-[var(--app-border)] px-5 py-4">{footer}</div>
        )}
      </aside>
    </>
  );
}

export function PanelSection({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <section className="space-y-2.5">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
        {label}
      </h3>
      {children}
    </section>
  );
}
