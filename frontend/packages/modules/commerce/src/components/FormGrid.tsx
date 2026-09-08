import type { ReactNode } from "react";

// egui: Grid::new(id).num_columns(2).min_col_width(110).spacing(Vec2::new(12,10))
// Tailwind equivalent: 2-col grid, label 110px, gap-x-3.5 gap-y-3
export function FormGrid({ children, id }: { children: ReactNode; id?: string }) {
  return (
    <div id={id} className="grid grid-cols-[110px_1fr] gap-x-3.5 gap-y-3 items-center">
      {children}
    </div>
  );
}

export function FormLabel({ children, required }: { children: ReactNode; required?: boolean }) {
  return (
    <span className="text-[11px] font-medium tracking-wide text-[var(--app-fg-muted)] uppercase">
      {children} {required && <span className="text-red-500">*</span>}
    </span>
  );
}

export function FormRow({ label, required, children }: { label: string; required?: boolean; children: ReactNode }) {
  return (
    <>
      <FormLabel required={required}>{label}</FormLabel>
      <div>{children}</div>
    </>
  );
}
