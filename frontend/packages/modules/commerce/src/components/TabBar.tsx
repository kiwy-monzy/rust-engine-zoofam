export function TabBar({ tabs, active, onChange }: { tabs: readonly string[]; active: number; onChange: (i: number) => void }) {
  return (
    <div className="flex gap-6 border-b border-[var(--app-border)] mb-4">
      {tabs.map((t, i) => (
        <button
          key={t}
          type="button"
          onClick={() => onChange(i)}
          className={`relative -mb-px px-1 py-3 text-sm font-medium transition-colors ${
            active === i ? "text-[var(--accent)]" : "text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]"
          }`}
        >
          {t}
          {active === i && <span className="absolute inset-x-0 -bottom-px h-0.5 bg-[var(--accent)]" />}
        </button>
      ))}
    </div>
  );
}
