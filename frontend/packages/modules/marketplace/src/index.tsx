import { useEffect, useRef, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router";
import { api } from "@gateway/lib";
import {
  Alert, Button, Card, EmptyState, Field, Input, PageHeader,
  SidePanel, Spinner, Table, Td, Textarea,
} from "@gateway/ui";

const cn = (...p: Array<string | false | null | undefined>) => p.filter(Boolean).join(" ");

type SubNav = { to: string; label: string; desc: string };
const MKT_NAV: SubNav[] = [
  { to: "/marketplace/organizations", label: "Organizations", desc: "Service providers" },
  { to: "/marketplace/categories", label: "Categories", desc: "Service categories" },
  { to: "/marketplace/services", label: "Services", desc: "Listed services" },
  { to: "/marketplace/bookings", label: "Bookings", desc: "All bookings" },
  { to: "/marketplace/requests", label: "Requests", desc: "Customer custom requests" },
  { to: "/marketplace/promotions", label: "Promotions", desc: "Discount codes" },
  { to: "/marketplace/notifications", label: "Notifications", desc: "User notifications" },
  { to: "/marketplace/payouts", label: "Payouts", desc: "Provider payouts" },
];

// ---------------------------------------------------------------- helpers ---

function useList<T>(url: string, key: string) {
  const [data, setData] = useState<T[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  async function load() {
    try { const r = await api.get<Record<string, T[]>>(url); setData((r as Record<string, T[]>)[key] ?? []); }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
  }
  useEffect(() => { void load(); }, [url, key]);
  return { data, err, reload: load };
}

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try { return await fn(); }
  catch (e) { alert(e instanceof Error ? e.message : String(e)); throw e; }
}

// ---------------------------------------------------------------- layout ---

export function MarketplaceLayout() {
  const loc = useLocation();
  return (
    <div className="flex gap-6">
      <aside className="hidden w-[210px] shrink-0 lg:block"
        style={{ position: "sticky", top: 80, alignSelf: "flex-start", height: "calc(100dvh - 96px)" }}>
        <div className="rounded-xl border border-[var(--app-border)] bg-[var(--app-card)] p-3">
          <div className="px-2 pb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">Marketplace</div>
          <nav className="flex flex-col gap-0.5">
            {MKT_NAV.map((i) => (
              <NavLink key={i.to} to={i.to}
                className={({ isActive }) => cn("rounded-lg px-3 py-2 text-sm transition-colors",
                  isActive ? "bg-[var(--sb-active-bg)] text-[var(--sb-active-fg)] font-semibold"
                    : "text-[var(--app-fg)] hover:bg-[var(--sb-hover)]")}>
                <div className="leading-tight">{i.label}</div>
                <div className="text-[10px] text-[var(--app-fg-muted)]">{i.desc}</div>
              </NavLink>
            ))}
          </nav>
        </div>
      </aside>
      <div className="min-w-0 flex-1 space-y-6">
        <div className="flex gap-2 overflow-x-auto pb-2 lg:hidden">
          {MKT_NAV.map((i) => (
            <NavLink
              key={i.to}
              to={i.to}
              className={({ isActive }) =>
                cn(
                  "whitespace-nowrap rounded-full border px-3 py-1.5 text-xs font-medium",
                  isActive
                    ? "bg-[var(--sb-active-bg)] text-[var(--sb-active-fg)] border-transparent"
                    : "bg-[var(--app-card)] text-[var(--app-fg-muted)] border-[var(--app-border)]",
                )
              }
            >
              {i.label}
            </NavLink>
          ))}
        </div>
        <Outlet />
        {loc.pathname === "/marketplace" && <MarketplaceOverview />}
      </div>
    </div>
  );
}

export function MarketplaceOverview() {
  const nav = useNavigate();
  return (
    <div className="space-y-6">
      <PageHeader title="Service Marketplace" description="Organizations → Services → Bookings → Requests → Promotions" />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {MKT_NAV.map((i) => (
          <Card key={i.to} className="cursor-pointer p-5 transition-shadow hover:shadow-md" onClick={() => nav(i.to)}>
            <div className="text-sm font-semibold">{i.label}</div>
            <div className="mt-1 text-xs text-[var(--app-fg-muted)]">{i.desc}</div>
            <div className="mt-3 text-xs text-[var(--accent-fg)]">Open →</div>
          </Card>
        ))}
      </div>
    </div>
  );
}

// ============================================================ Organizations ===

type Organization = {
  id: string; owner_user_id: string; name: string; slug: string;
  business_name?: string; description?: string; phone?: string; email?: string;
  city?: string; country?: string; is_verified: boolean; is_active: boolean;
  rating_avg: number; rating_count: number; commission_rate: number;
  created_at: string;
};

type PickerUser = { id: string; email: string; display_name: string; username: string };

function UserPicker({
  value,
  onChange,
  label = "Owner",
}: {
  value: string;
  onChange: (userId: string) => void;
  label?: string;
}) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<PickerUser[]>([]);
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (wrapRef.current && !wrapRef.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  function search(q: string) {
    setQuery(q);
    if (timer.current) clearTimeout(timer.current);
    if (!q.trim()) { setResults([]); setOpen(false); return; }
    timer.current = setTimeout(async () => {
      setBusy(true);
      try {
        const res = await api.get<{ users: PickerUser[] }>(`/users/search?q=${encodeURIComponent(q.trim())}&limit=8`);
        setResults(res.users);
        setOpen(true);
      } catch { setResults([]); }
      finally { setBusy(false); }
    }, 250);
  }

  const display = results.find((u) => u.id === value);

  return (
    <Field label={label}>
      <div ref={wrapRef} className="relative">
        <Input
          value={display ? `${display.display_name || display.email} (${display.username || display.id.slice(0, 8)})` : query}
          onChange={(e) => { search(e.target.value); onChange(""); }}
          onFocus={() => { if (results.length > 0) setOpen(true); }}
          placeholder="Search by name, email or username…"
        />
        {busy && <span className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-[var(--app-fg-muted)]">…</span>}
        {open && results.length > 0 && (
          <div className="absolute left-0 top-full z-50 mt-1 max-h-60 w-full overflow-y-auto rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] shadow-lg">
            {results.map((u) => (
              <button
                key={u.id}
                type="button"
                className={`flex w-full items-center gap-2 px-3 py-2 text-left text-sm hover:bg-[var(--sb-hover)] ${u.id === value ? "bg-[var(--accent-soft)]" : ""}`}
                onMouseDown={() => { onChange(u.id); setQuery(""); setOpen(false); }}
              >
                <span className="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-[var(--app-card-2)] text-xs font-bold text-[var(--app-fg-muted)]">
                  {(u.display_name || u.email).charAt(0).toUpperCase()}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium text-[var(--app-fg)]">{u.display_name || u.email}</span>
                  <span className="block truncate text-xs text-[var(--app-fg-muted)]">{u.username ? `@${u.username}` : u.email}</span>
                </span>
              </button>
            ))}
          </div>
        )}
      </div>
    </Field>
  );
}

export function OrganizationsPage() {
  const { data, err, reload } = useList<Organization>("/marketplace/organizations", "organizations");
  const nav = useNavigate();
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState({ owner_user_id: "", name: "", slug: "", business_name: "", description: "", phone: "", email: "", city: "", country: "", base_currency: "KES" });
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Organizations" description="Service providers on the marketplace"
        actions={<Button onClick={() => setOpen(true)}>Add Organization</Button>} />
      {data.length === 0 ? <EmptyState message="No organizations yet" /> : (
        <Table head={["Name", "Slug", "City", "Rating", "Status", ""]}>
          {data.map((o) => (
            <tr key={o.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td>{o.name}</Td>
              <Td className="font-mono text-xs">{o.slug}</Td>
              <Td>{o.city || "—"}</Td>
              <Td>{o.rating_avg.toFixed(1)} ({o.rating_count})</Td>
              <Td>{o.is_verified ? <span className="inline-flex items-center rounded-full bg-emerald-500/15 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:text-emerald-300">Verified</span> : <span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)]">{o.is_active ? "Active" : "Inactive"}</span>}</Td>
              <Td className="text-right"><Button variant="ghost" onClick={() => nav(`/marketplace/organizations/${o.id}`)}>View</Button></Td>
            </tr>
          ))}
        </Table>
      )}
      <SidePanel open={open} onClose={() => setOpen(false)} title="New Organization">
        <div className="space-y-3">
          <UserPicker value={form.owner_user_id} onChange={(id) => setForm({ ...form, owner_user_id: id })} />
          <Field label="Name"><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></Field>
          <Field label="Slug"><Input value={form.slug} onChange={(e) => setForm({ ...form, slug: e.target.value })} /></Field>
          <Field label="Business Name"><Input value={form.business_name} onChange={(e) => setForm({ ...form, business_name: e.target.value })} /></Field>
          <Field label="Phone"><Input value={form.phone} onChange={(e) => setForm({ ...form, phone: e.target.value })} /></Field>
          <Field label="Email"><Input value={form.email} onChange={(e) => setForm({ ...form, email: e.target.value })} /></Field>
          <Field label="City"><Input value={form.city} onChange={(e) => setForm({ ...form, city: e.target.value })} /></Field>
          <Field label="Country"><Input value={form.country} onChange={(e) => setForm({ ...form, country: e.target.value })} /></Field>
          <Field label="Description"><Textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} /></Field>
          <Button onClick={async () => {
            await call(() => api.post("/marketplace/organizations", form));
            setOpen(false); setForm({ owner_user_id: "", name: "", slug: "", business_name: "", description: "", phone: "", email: "", city: "", country: "", base_currency: "KES" });
            reload();
          }}>Create</Button>
        </div>
      </SidePanel>
    </div>
  );
}

export function OrganizationDetailPage({ id }: { id: string }) {
  const [org, setOrg] = useState<Organization | null>(null);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    api.get<{ organization: Organization }>(`/marketplace/organizations/${id}`)
      .then((r) => setOrg(r.organization))
      .catch((e) => setErr(e instanceof Error ? e.message : String(e)));
  }, [id]);
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!org) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title={org.name} description={org.business_name || org.slug} />
      <Card className="p-5 space-y-2">
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div><span className="text-[var(--app-fg-muted)]">Email:</span> {org.email || "—"}</div>
          <div><span className="text-[var(--app-fg-muted)]">Phone:</span> {org.phone || "—"}</div>
          <div><span className="text-[var(--app-fg-muted)]">City:</span> {org.city || "—"}</div>
          <div><span className="text-[var(--app-fg-muted)]">Country:</span> {org.country || "—"}</div>
          <div><span className="text-[var(--app-fg-muted)]">Rating:</span> {org.rating_avg.toFixed(1)} ({org.rating_count} reviews)</div>
          <div><span className="text-[var(--app-fg-muted)]">Commission:</span> {(org.commission_rate * 100).toFixed(0)}%</div>
          <div><span className="text-[var(--app-fg-muted)]">Verified:</span> {org.is_verified ? "Yes" : "No"}</div>
          <div><span className="text-[var(--app-fg-muted)]">Active:</span> {org.is_active ? "Yes" : "No"}</div>
        </div>
        {org.description && <div className="mt-3 text-sm text-[var(--app-fg-muted)]">{org.description}</div>}
      </Card>
    </div>
  );
}

// ============================================================ Categories ===

type Category = { id: string; name: string; slug: string; description?: string; sort_order: number; is_active: boolean };

export function CategoriesPage() {
  const { data, err, reload } = useList<Category>("/marketplace/categories", "categories");
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState({ name: "", slug: "", description: "", sort_order: 0 });
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Categories" description="Service categories for the marketplace"
        actions={<Button onClick={() => setOpen(true)}>Add Category</Button>} />
      {data.length === 0 ? <EmptyState message="No categories yet" /> : (
        <Table head={["Name", "Slug", "Sort", "Status", ""]}>
          {data.map((c) => (
            <tr key={c.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td>{c.name}</Td>
              <Td className="font-mono text-xs">{c.slug}</Td>
              <Td>{c.sort_order}</Td>
              <Td>{c.is_active ? <span className="inline-flex items-center rounded-full bg-emerald-500/15 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:text-emerald-300">Active</span> : <span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)]">Inactive</span>}</Td>
              <Td className="text-right">
                <Button variant="ghost" onClick={async () => {
                  if (confirm(`Delete category "${c.name}"?`)) {
                    await call(() => api.del(`/marketplace/categories/${c.id}`));
                    reload();
                  }
                }}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <SidePanel open={open} onClose={() => setOpen(false)} title="New Category">
        <div className="space-y-3">
          <Field label="Name"><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></Field>
          <Field label="Slug"><Input value={form.slug} onChange={(e) => setForm({ ...form, slug: e.target.value })} /></Field>
          <Field label="Sort Order"><Input type="number" value={form.sort_order} onChange={(e) => setForm({ ...form, sort_order: +e.target.value })} /></Field>
          <Field label="Description"><Textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} /></Field>
          <Button onClick={async () => {
            await call(() => api.post("/marketplace/categories", form));
            setOpen(false); setForm({ name: "", slug: "", description: "", sort_order: 0 });
            reload();
          }}>Create</Button>
        </div>
      </SidePanel>
    </div>
  );
}

// ============================================================ Services ===

type Service = {
  id: string; org_id: string; category_id?: string; name: string; slug: string;
  description?: string; price_type: string; base_price: number; currency: string;
  duration_minutes?: number; is_active: boolean; rating_avg: number; rating_count: number;
  booking_count: number; created_at: string;
};

export function ServicesPage() {
  const { data, err, reload } = useList<Service>("/marketplace/services", "services");
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState({ org_id: "", name: "", slug: "", description: "", price_type: "fixed", base_price: 0, currency: "KES", duration_minutes: "" });
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Services" description="All marketplace services"
        actions={<Button onClick={() => setOpen(true)}>Add Service</Button>} />
      {data.length === 0 ? <EmptyState message="No services yet" /> : (
        <Table head={["Name", "Org ID", "Price", "Type", "Bookings", "Status", ""]}>
          {data.map((s) => (
            <tr key={s.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td>{s.name}</Td>
              <Td className="font-mono text-xs">{s.org_id.slice(0, 8)}…</Td>
              <Td>{s.base_price} {s.currency}</Td>
              <Td><span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)] border border-[var(--app-border)]">{s.price_type}</span></Td>
              <Td>{s.booking_count}</Td>
              <Td>{s.is_active ? <span className="inline-flex items-center rounded-full bg-emerald-500/15 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:text-emerald-300">Active</span> : <span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)]">Inactive</span>}</Td>
              <Td className="text-right">
                <Button variant="ghost" onClick={async () => {
                  if (confirm(`Delete service "${s.name}"?`)) {
                    await call(() => api.del(`/marketplace/services/${s.id}`));
                    reload();
                  }
                }}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <SidePanel open={open} onClose={() => setOpen(false)} title="New Service">
        <div className="space-y-3">
          <Field label="Organization ID"><Input value={form.org_id} onChange={(e) => setForm({ ...form, org_id: e.target.value })} /></Field>
          <Field label="Name"><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} /></Field>
          <Field label="Slug"><Input value={form.slug} onChange={(e) => setForm({ ...form, slug: e.target.value })} /></Field>
          <Field label="Price Type">
            <select className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
              value={form.price_type} onChange={(e) => setForm({ ...form, price_type: e.target.value })}>
              <option value="fixed">Fixed</option>
              <option value="hourly">Hourly</option>
              <option value="daily">Daily</option>
              <option value="custom">Custom</option>
            </select>
          </Field>
          <Field label="Base Price"><Input type="number" value={form.base_price} onChange={(e) => setForm({ ...form, base_price: +e.target.value })} /></Field>
          <Field label="Duration (min)"><Input type="number" value={form.duration_minutes} onChange={(e) => setForm({ ...form, duration_minutes: e.target.value })} /></Field>
          <Field label="Description"><Textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} /></Field>
          <Button onClick={async () => {
            await call(() => api.post("/marketplace/services", { ...form, duration_minutes: form.duration_minutes ? +form.duration_minutes : undefined }));
            setOpen(false); setForm({ org_id: "", name: "", slug: "", description: "", price_type: "fixed", base_price: 0, currency: "KES", duration_minutes: "" });
            reload();
          }}>Create</Button>
        </div>
      </SidePanel>
    </div>
  );
}

// ============================================================ Bookings ===

type Booking = {
  id: string; org_id: string; service_id: string; customer_user_id: string;
  booking_type: string; status: string; total_price: number; currency: string;
  payment_status: string; created_at: string;
};

export function BookingsPage() {
  const { data, err } = useList<Booking>("/marketplace/bookings", "bookings");
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Bookings" description="All marketplace bookings" />
      {data.length === 0 ? <EmptyState message="No bookings yet" /> : (
        <Table head={["ID", "Type", "Status", "Price", "Payment", "Created"]}>
          {data.map((b) => (
            <tr key={b.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{b.id.slice(0, 8)}…</Td>
              <Td><span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)] border border-[var(--app-border)]">{b.booking_type}</span></Td>
              <Td><span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${b.status === "completed" ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300" : b.status === "cancelled" ? "bg-red-500/15 text-red-700 dark:text-red-300" : "bg-[var(--app-card-2)] text-[var(--app-fg-muted)] border border-[var(--app-border)]"}`}>{b.status}</span></Td>
              <Td>{b.total_price} {b.currency}</Td>
              <Td><span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${b.payment_status === "paid" ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300" : "bg-[var(--app-card-2)] text-[var(--app-fg-muted)] border border-[var(--app-border)]"}`}>{b.payment_status}</span></Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">{new Date(b.created_at).toLocaleDateString()}</Td>
            </tr>
          ))}
        </Table>
      )}
    </div>
  );
}

// ============================================================ Requests ===

type Request = {
  id: string; customer_user_id: string; title: string; description?: string;
  budget_min?: number; budget_max?: number; currency: string;
  status: string; created_at: string;
};

export function RequestsPage() {
  const { data, err } = useList<Request>("/marketplace/requests", "requests");
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Requests" description="Customer custom service requests" />
      {data.length === 0 ? <EmptyState message="No requests yet" /> : (
        <Table head={["Title", "Budget", "Status", "Created"]}>
          {data.map((r) => (
            <tr key={r.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td>{r.title}</Td>
              <Td>{r.budget_min || "—"} – {r.budget_max || "—"} {r.currency}</Td>
              <Td><span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${r.status === "open" ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300" : "bg-[var(--app-card-2)] text-[var(--app-fg-muted)] border border-[var(--app-border)]"}`}>{r.status}</span></Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">{new Date(r.created_at).toLocaleDateString()}</Td>
            </tr>
          ))}
        </Table>
      )}
    </div>
  );
}

// ============================================================ Promotions ===

type Promotion = {
  id: string; code: string; description?: string; discount_type: string;
  discount_value: number; max_uses?: number; used_count: number;
  is_active: boolean; created_at: string;
};

export function PromotionsPage() {
  const { data, err, reload } = useList<Promotion>("/marketplace/promotions", "promotions");
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState({ code: "", description: "", discount_type: "percent", discount_value: 0, max_uses: "", starts_at: "", expires_at: "" });
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Promotions" description="Discount codes for the marketplace"
        actions={<Button onClick={() => setOpen(true)}>Add Promotion</Button>} />
      {data.length === 0 ? <EmptyState message="No promotions yet" /> : (
        <Table head={["Code", "Type", "Value", "Uses", "Status", ""]}>
          {data.map((p) => (
            <tr key={p.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{p.code}</Td>
              <Td><span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)] border border-[var(--app-border)]">{p.discount_type}</span></Td>
              <Td>{p.discount_value}{p.discount_type === "percent" ? "%" : ""}</Td>
              <Td>{p.used_count}{p.max_uses ? ` / ${p.max_uses}` : ""}</Td>
              <Td>{p.is_active ? <span className="inline-flex items-center rounded-full bg-emerald-500/15 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:text-emerald-300">Active</span> : <span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)]">Inactive</span>}</Td>
              <Td className="text-right">
                <Button variant="ghost" onClick={async () => {
                  if (confirm(`Delete promotion "${p.code}"?`)) {
                    await call(() => api.del(`/marketplace/promotions/${p.id}`));
                    reload();
                  }
                }}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <SidePanel open={open} onClose={() => setOpen(false)} title="New Promotion">
        <div className="space-y-3">
          <Field label="Code"><Input value={form.code} onChange={(e) => setForm({ ...form, code: e.target.value })} /></Field>
          <Field label="Discount Type">
            <select className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
              value={form.discount_type} onChange={(e) => setForm({ ...form, discount_type: e.target.value })}>
              <option value="percent">Percent</option>
              <option value="fixed">Fixed Amount</option>
            </select>
          </Field>
          <Field label="Discount Value"><Input type="number" value={form.discount_value} onChange={(e) => setForm({ ...form, discount_value: +e.target.value })} /></Field>
          <Field label="Max Uses"><Input type="number" value={form.max_uses} onChange={(e) => setForm({ ...form, max_uses: e.target.value })} /></Field>
          <Field label="Starts At"><Input type="datetime-local" value={form.starts_at} onChange={(e) => setForm({ ...form, starts_at: e.target.value })} /></Field>
          <Field label="Expires At"><Input type="datetime-local" value={form.expires_at} onChange={(e) => setForm({ ...form, expires_at: e.target.value })} /></Field>
          <Field label="Description"><Textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} /></Field>
          <Button onClick={async () => {
            const toServer = (v: string) => v ? new Date(v).toISOString().slice(0, 19) : "";
            await call(() => api.post("/marketplace/promotions", {
              ...form, max_uses: form.max_uses ? +form.max_uses : undefined,
              starts_at: toServer(form.starts_at), expires_at: toServer(form.expires_at),
            }));
            setOpen(false); setForm({ code: "", description: "", discount_type: "percent", discount_value: 0, max_uses: "", starts_at: "", expires_at: "" });
            reload();
          }}>Create</Button>
        </div>
      </SidePanel>
    </div>
  );
}

// ============================================================ Notifications ===

type Notification = { id: string; user_id: string; kind: string; title: string; body?: string; is_read: boolean; created_at: string };

export function NotificationsPage() {
  const { data, err } = useList<Notification>("/marketplace/notifications/me", "notifications");
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Notifications" description="Marketplace notifications" />
      {data.length === 0 ? <EmptyState message="No notifications" /> : (
        <Table head={["Kind", "Title", "Read", "Created"]}>
          {data.map((n) => (
            <tr key={n.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td><span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)] border border-[var(--app-border)]">{n.kind}</span></Td>
              <Td>{n.title}</Td>
              <Td>{n.is_read ? <span className="inline-flex items-center rounded-full bg-emerald-500/15 px-2 py-0.5 text-xs font-medium text-emerald-700 dark:text-emerald-300">Read</span> : <span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)] border border-[var(--app-border)]">Unread</span>}</Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">{new Date(n.created_at).toLocaleDateString()}</Td>
            </tr>
          ))}
        </Table>
      )}
    </div>
  );
}

// ============================================================ Payouts ===

type Payout = { id: string; org_id: string; amount: number; currency: string; method: string; status: string; created_at: string };

export function PayoutsPage() {
  const { data, err } = useList<Payout>("/marketplace/payouts", "payouts");
  if (err) return <Alert tone="error">{err}</Alert>;
  if (!data) return <Spinner />;
  return (
    <div className="space-y-4">
      <PageHeader title="Payouts" description="Provider payout history" />
      {data.length === 0 ? <EmptyState message="No payouts yet" /> : (
        <Table head={["Org ID", "Amount", "Method", "Status", "Created"]}>
          {data.map((p) => (
            <tr key={p.id} className="border-t border-[var(--app-border)] hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{p.org_id.slice(0, 8)}…</Td>
              <Td>{p.amount} {p.currency}</Td>
              <Td><span className="inline-flex items-center rounded-full bg-[var(--app-card-2)] px-2 py-0.5 text-xs font-medium text-[var(--app-fg-muted)] border border-[var(--app-border)]">{p.method}</span></Td>
              <Td><span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${p.status === "completed" ? "bg-emerald-500/15 text-emerald-700 dark:text-emerald-300" : "bg-[var(--app-card-2)] text-[var(--app-fg-muted)] border border-[var(--app-border)]"}`}>{p.status}</span></Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">{new Date(p.created_at).toLocaleDateString()}</Td>
            </tr>
          ))}
        </Table>
      )}
    </div>
  );
}
