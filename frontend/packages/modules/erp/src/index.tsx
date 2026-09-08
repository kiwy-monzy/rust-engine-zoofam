// ERP module — Products, Units, Warehouses, Suppliers, Procurement,
// Purchase Orders, Goods Receipts, Inventory, Sales Orders, Shipments,
// Assets, Expenses, Invoices.
import { useEffect, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router";
import { api } from "@gateway/lib";
import {
  Alert,
  Badge,
  Button,
  Card,
  EmptyState,
  Field,
  Input,
  PageHeader,
  SidePanel,
  Spinner,
  Table,
  Td,
  Textarea,
} from "@gateway/ui";

const cn = (...parts: Array<string | false | null | undefined>) =>
  parts.filter(Boolean).join(" ");

type SubNav = { to: string; label: string; desc: string };
const ERP_NAV: SubNav[] = [
  { to: "/erp/products", label: "Products", desc: "SKU · catalog" },
  { to: "/erp/units", label: "Units", desc: "Measurements" },
  { to: "/erp/warehouses", label: "Warehouses", desc: "Locations" },
  { to: "/erp/suppliers", label: "Suppliers", desc: "Vendors" },
  { to: "/erp/procurement", label: "Procurement", desc: "Requests + lines" },
  { to: "/erp/purchase-orders", label: "Purchase Orders", desc: "POs + lines" },
  { to: "/erp/goods-receipts", label: "Goods Receipts", desc: "Inbound + ledger" },
  { to: "/erp/inventory", label: "Inventory", desc: "Stock & ledger" },
  { to: "/erp/sales-orders", label: "Sales Orders", desc: "SO + lines" },
  { to: "/erp/shipments", label: "Shipments", desc: "Outbound −qty" },
  { to: "/erp/assets", label: "Assets", desc: "Capitalized" },
  { to: "/erp/expenses", label: "Expenses", desc: "Spend" },
  { to: "/erp/invoices", label: "Invoices", desc: "Accounting" },
];

// ----------------------------------------------------- layout / shell --

export function ErpLayout() {
  const loc = useLocation();
  return (
    <div className="flex gap-6" style={{ marginLeft: 0 }}>
      <aside
        className="hidden w-[210px] shrink-0 lg:block"
        style={{ position: "sticky", top: 80, alignSelf: "flex-start", height: "calc(100dvh - 96px)" }}
      >
        <div className="rounded-xl border border-[var(--app-border)] bg-[var(--app-card)] p-3">
          <div className="px-2 pb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            ERP
          </div>
          <nav className="flex flex-col gap-0.5">
            {ERP_NAV.map((i) => (
              <NavLink
                key={i.to}
                to={i.to}
                className={({ isActive }) =>
                  cn(
                    "rounded-lg px-3 py-2 text-sm transition-colors",
                    isActive
                      ? "bg-[var(--sb-active-bg)] text-[var(--sb-active-fg)] font-semibold"
                      : "text-[var(--app-fg)] hover:bg-[var(--sb-hover)]",
                  )
                }
              >
                <div className="leading-tight">{i.label}</div>
                <div className="text-[10px] text-[var(--app-fg-muted)]">{i.desc}</div>
              </NavLink>
            ))}
          </nav>
        </div>
      </aside>
      <div className="min-w-0 flex-1 space-y-6">
        <div className="flex gap-2 overflow-x-auto pb-2 lg:hidden">
          {ERP_NAV.map((i) => (
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
        {loc.pathname === "/erp" && <ErpOverview />}
      </div>
    </div>
  );
}

export function ErpOverview() {
  const nav = useNavigate();
  return (
    <div className="space-y-6">
      <PageHeader
        title="ERP"
        description="Products → Procurement → Purchasing → Goods Receipt → Inventory → Sales / Assets → Accounting"
      />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {ERP_NAV.map((i) => (
          <Card
            key={i.to}
            className="cursor-pointer p-5 transition-shadow hover:shadow-md"
            onClick={() => nav(i.to)}
          >
            <div className="text-sm font-semibold">{i.label}</div>
            <div className="mt-1 text-xs text-[var(--app-fg-muted)]">{i.desc}</div>
            <div className="mt-3 text-xs text-[var(--accent-fg)]">Open →</div>
          </Card>
        ))}
      </div>
    </div>
  );
}

// --------------------------------------------------------- helpers --

function useList<T>(url: string, key: string) {
  const [data, setData] = useState<T[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  async function load() {
    try {
      const r = await api.get<Record<string, T[]>>(url);
      setData((r as Record<string, T[]>)[key] ?? []);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => {
    void load();
  }, [url, key]);
  return { data, err, reload: load };
}

function todayIso() {
  return new Date().toISOString().slice(0, 10);
}

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    alert(e instanceof Error ? e.message : String(e));
    throw e;
  }
}

// ============================================================================
// Products
// ============================================================================

type Product = {
  id: string;
  sku: string;
  name: string;
  description?: string | null;
  unit_id?: string | null;
  product_type: string;
  is_active: boolean;
};

export function ProductsPage() {
  const { data, err, reload } = useList<Product>("/erp/products", "products");
  const units = useList<{ id: string; name: string }>("/erp/units", "units");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Product | null>(null);

  async function create(body: Partial<Product>) {
    await call(() => api.post<Product>("/erp/products", { id: "", is_active: true, ...body }));
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Product>) {
    await call(() => api.patch<Product>(`/erp/products/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this product?")) return;
    await call(() => api.del<void>(`/erp/products/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Products"
        description="Catalog — SKU is unique; stock comes from the ledger, not a mutable column."
        actions={<Button onClick={() => setOpen(true)}>New product</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No products yet." />
      ) : (
        <Table head={["SKU", "Name", "Type", "Active", ""]}>
          {data.map((p) => (
            <tr key={p.id} className="hover:bg-[var(--sb-hover)]">
              <Td>
                <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 text-xs">{p.sku}</code>
              </Td>
              <Td className="font-medium">{p.name}</Td>
              <Td>
                <Badge>{p.product_type}</Badge>
              </Td>
              <Td>
                <Badge tone={p.is_active ? "green" : "zinc"}>{String(p.is_active)}</Badge>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(p)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(p.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ProductForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        units={units.data ?? []}
        title="New product"
      />
      <ProductForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        units={units.data ?? []}
        title={`Edit product ${edit?.sku ?? ""}`}
      />
    </div>
  );
}

function ProductForm({
  open,
  initial,
  onClose,
  onSubmit,
  units,
  title,
}: {
  open: boolean;
  initial?: Product;
  onClose: () => void;
  onSubmit: (b: Partial<Product>) => Promise<void> | void;
  units: { id: string; name: string }[];
  title: string;
}) {
  const [sku, setSku] = useState(initial?.sku ?? "");
  const [name, setName] = useState(initial?.name ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [unitId, setUnitId] = useState(initial?.unit_id ?? "");
  const [type, setType] = useState(initial?.product_type ?? "goods");
  const [active, setActive] = useState(initial?.is_active ?? true);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setSku(initial?.sku ?? "");
      setName(initial?.name ?? "");
      setDescription(initial?.description ?? "");
      setUnitId(initial?.unit_id ?? "");
      setType(initial?.product_type ?? "goods");
      setActive(initial?.is_active ?? true);
    }
  }, [open, initial]);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ sku, name, description, unit_id: unitId || null, product_type: type, is_active: active });
    } finally {
      setBusy(false);
    }
  }

  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="SKU">
          <Input required value={sku} onChange={(e) => setSku(e.target.value)} placeholder="SKU-..." />
        </Field>
        <Field label="Name">
          <Input required value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Description">
          <Textarea rows={3} value={description} onChange={(e) => setDescription(e.target.value)} />
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Unit">
            <select
              value={unitId}
              onChange={(e) => setUnitId(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              <option value="">— none —</option>
              {units.map((u) => (
                <option key={u.id} value={u.id}>
                  {u.name}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Type">
            <select
              value={type}
              onChange={(e) => setType(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              <option value="goods">goods</option>
              <option value="service">service</option>
              <option value="consumable">consumable</option>
            </select>
          </Field>
        </div>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={active} onChange={(e) => setActive(e.target.checked)} />
          Active
        </label>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Units
// ============================================================================

type Unit = { id: string; name: string; symbol: string };

export function UnitsPage() {
  const { data, err, reload } = useList<Unit>("/erp/units", "units");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Unit | null>(null);

  async function create(body: Partial<Unit>) {
    await call(() => api.post<Unit>("/erp/units", { id: "", ...body }));
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Unit>) {
    await call(() => api.patch<Unit>(`/erp/units/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this unit?")) return;
    await call(() => api.del<void>(`/erp/units/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Units"
        description="Units of measure used by products (kg, pcs, l…)."
        actions={<Button onClick={() => setOpen(true)}>New unit</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No units yet." />
      ) : (
        <Table head={["Name", "Symbol", ""]}>
          {data.map((u) => (
            <tr key={u.id}>
              <Td className="font-medium">{u.name}</Td>
              <Td>
                <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 text-xs">{u.symbol}</code>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(u)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(u.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <UnitForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New unit"
      />
      <UnitForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit unit ${edit?.name ?? ""}`}
      />
    </div>
  );
}

function UnitForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: Unit;
  onClose: () => void;
  onSubmit: (b: Partial<Unit>) => Promise<void> | void;
  title: string;
}) {
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setName(initial?.name ?? "");
      setSymbol(initial?.symbol ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ name, symbol });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name">
          <Input required value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Symbol">
          <Input required value={symbol} onChange={(e) => setSymbol(e.target.value)} placeholder="pcs / kg / l" />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Warehouses
// ============================================================================

type Warehouse = { id: string; name: string; location?: string | null };

export function WarehousesPage() {
  const { data, err, reload } = useList<Warehouse>("/erp/warehouses", "warehouses");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Warehouse | null>(null);

  async function create(body: Partial<Warehouse>) {
    await call(() => api.post<Warehouse>("/erp/warehouses", { id: "", ...body }));
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Warehouse>) {
    await call(() => api.patch<Warehouse>(`/erp/warehouses/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this warehouse?")) return;
    await call(() => api.del<void>(`/erp/warehouses/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Warehouses"
        description="Storage locations. Stock ledger groups by (product, warehouse)."
        actions={<Button onClick={() => setOpen(true)}>New warehouse</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No warehouses yet." />
      ) : (
        <Table head={["Name", "Location", ""]}>
          {data.map((w) => (
            <tr key={w.id}>
              <Td className="font-medium">{w.name}</Td>
              <Td className="text-sm text-[var(--app-fg-muted)]">{w.location ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(w)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(w.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <WarehouseForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New warehouse"
      />
      <WarehouseForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit warehouse ${edit?.name ?? ""}`}
      />
    </div>
  );
}

function WarehouseForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: Warehouse;
  onClose: () => void;
  onSubmit: (b: Partial<Warehouse>) => Promise<void> | void;
  title: string;
}) {
  const [name, setName] = useState("");
  const [location, setLocation] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setName(initial?.name ?? "");
      setLocation(initial?.location ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ name, location });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name">
          <Input required value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Location">
          <Input value={location} onChange={(e) => setLocation(e.target.value)} placeholder="City, address" />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Suppliers
// ============================================================================

type Supplier = {
  id: string;
  name: string;
  contact_email?: string | null;
  contact_phone?: string | null;
};

export function SuppliersPage() {
  const { data, err, reload } = useList<Supplier>("/erp/suppliers", "suppliers");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Supplier | null>(null);

  async function create(body: Partial<Supplier>) {
    await call(() => api.post<Supplier>("/erp/suppliers", { id: "", ...body }));
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Supplier>) {
    await call(() => api.patch<Supplier>(`/erp/suppliers/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this supplier?")) return;
    await call(() => api.del<void>(`/erp/suppliers/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Suppliers"
        description="Vendors for purchase orders."
        actions={<Button onClick={() => setOpen(true)}>New supplier</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No suppliers yet." />
      ) : (
        <Table head={["Name", "Email", "Phone", ""]}>
          {data.map((s) => (
            <tr key={s.id}>
              <Td className="font-medium">{s.name}</Td>
              <Td className="text-xs">{s.contact_email ?? "—"}</Td>
              <Td className="text-xs">{s.contact_phone ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(s)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(s.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <SupplierForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New supplier"
      />
      <SupplierForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit supplier ${edit?.name ?? ""}`}
      />
    </div>
  );
}

function SupplierForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: Supplier;
  onClose: () => void;
  onSubmit: (b: Partial<Supplier>) => Promise<void> | void;
  title: string;
}) {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [phone, setPhone] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setName(initial?.name ?? "");
      setEmail(initial?.contact_email ?? "");
      setPhone(initial?.contact_phone ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ name, contact_email: email || null, contact_phone: phone || null });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name">
          <Input required value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Email">
          <Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
        </Field>
        <Field label="Phone">
          <Input value={phone} onChange={(e) => setPhone(e.target.value)} />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Procurement Requests + Lines
// ============================================================================

type Procurement = {
  id: string;
  request_number: string;
  requested_by?: string | null;
  status: string;
  notes?: string | null;
};
type ProcurementLine = {
  id: string;
  procurement_id: string;
  product_id: string;
  quantity: number;
  notes?: string | null;
};

export function ProcurementPage() {
  const { data, err, reload } = useList<Procurement>("/erp/procurement", "requests");
  const [open, setOpen] = useState(false);
  const [picked, setPicked] = useState<Procurement | null>(null);
  const [edit, setEdit] = useState<Procurement | null>(null);

  async function create(body: Partial<Procurement>) {
    await call(() => api.post<Procurement>("/erp/procurement", { id: "", request_number: "", status: "DRAFT", ...body }));
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Procurement>) {
    await call(() => api.patch<Procurement>(`/erp/procurement/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this request?")) return;
    await call(() => api.del<void>(`/erp/procurement/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Procurement"
        description="DRAFT → SUBMITTED → APPROVED → PO_CREATED"
        actions={<Button onClick={() => setOpen(true)}>New request</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No procurement requests." />
      ) : (
        <Table head={["Request #", "Status", "Notes", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{r.request_number}</Td>
              <Td>
                <Badge>{r.status}</Badge>
              </Td>
              <Td className="max-w-[280px] truncate text-xs text-[var(--app-fg-muted)]">{r.notes ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setPicked(r)}>
                  Lines
                </Button>
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ProcurementForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New procurement request"
      />
      <ProcurementForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit ${edit?.request_number ?? ""}`}
      />
      <ProcurementLinesPanel
        open={!!picked}
        request={picked}
        onClose={() => setPicked(null)}
      />
    </div>
  );
}

function ProcurementForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: Procurement;
  onClose: () => void;
  onSubmit: (b: Partial<Procurement>) => Promise<void> | void;
  title: string;
}) {
  const [status, setStatus] = useState("DRAFT");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setStatus(initial?.status ?? "DRAFT");
      setNotes(initial?.notes ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ status, notes });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Status">
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {["DRAFT", "SUBMITTED", "APPROVED", "PO_CREATED", "REJECTED"].map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Notes">
          <Textarea rows={4} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

function ProcurementLinesPanel({
  open,
  request,
  onClose,
}: {
  open: boolean;
  request: Procurement | null;
  onClose: () => void;
}) {
  const [lines, setLines] = useState<ProcurementLine[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const products = useList<Product>("/erp/products", "products");

  async function load() {
    if (!request) return;
    try {
      const r = await api.get<{ lines: ProcurementLine[] }>(`/erp/procurement/${request.id}/lines`);
      setLines(r.lines);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => {
    if (open && request) void load();
  }, [open, request]);

  async function addLine(productId: string, qty: number) {
    if (!request) return;
    await call(() =>
      api.post<ProcurementLine>(`/erp/procurement/${request.id}/lines`, {
        id: "",
        procurement_id: request.id,
        product_id: productId,
        quantity: qty,
        notes: null,
      }),
    );
    await load();
  }
  async function del(id: string) {
    if (!request) return;
    if (!confirm("Delete line?")) return;
    await call(() => api.del<void>(`/erp/procurement/${request.id}/lines/${id}`));
    await load();
  }

  return (
    <SidePanel
      open={open}
      onClose={onClose}
      title={`Lines — ${request?.request_number ?? ""}`}
      width={520}
    >
      {err && <Alert>{err}</Alert>}
      <div className="space-y-2">
        {!lines ? (
          <Spinner />
        ) : lines.length === 0 ? (
          <EmptyState message="No lines yet." />
        ) : (
          <Table head={["Product", "Qty", ""]}>
            {lines.map((l) => (
              <tr key={l.id}>
                <Td className="font-mono text-xs">{l.product_id.slice(0, 8)}</Td>
                <Td>{l.quantity}</Td>
                <Td className="text-right">
                  <Button variant="danger" className="py-1 text-xs" onClick={() => del(l.id)}>
                    Remove
                  </Button>
                </Td>
              </tr>
            ))}
          </Table>
        )}
      </div>
      <div className="flex justify-end">
        <Button onClick={() => setAdding(true)}>Add line</Button>
      </div>
      <AddProcurementLinePanel
        open={adding}
        onClose={() => setAdding(false)}
        products={products.data ?? []}
        onAdd={addLine}
      />
    </SidePanel>
  );
}

function AddProcurementLinePanel({
  open,
  onClose,
  products,
  onAdd,
}: {
  open: boolean;
  onClose: () => void;
  products: { id: string; sku: string; name: string }[];
  onAdd: (productId: string, qty: number) => Promise<void> | void;
}) {
  const [productId, setProductId] = useState("");
  const [qty, setQty] = useState(1);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setProductId(products[0]?.id ?? "");
      setQty(1);
    }
  }, [open, products]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!productId) return;
    setBusy(true);
    try {
      await onAdd(productId, qty);
      onClose();
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="Add line" width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Product">
          <select
            required
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {products.map((p) => (
              <option key={p.id} value={p.id}>
                {p.sku} — {p.name}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Quantity">
          <Input
            type="number"
            min={0}
            step="any"
            value={qty}
            onChange={(e) => setQty(Number(e.target.value))}
          />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Add
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Purchase Orders + Lines
// ============================================================================

type PO = {
  id: string;
  po_number: string;
  supplier_id?: string | null;
  status: string;
  notes?: string | null;
};
type POLine = {
  id: string;
  purchase_order_id: string;
  product_id: string;
  quantity: number;
  unit_price: number;
  notes?: string | null;
};

export function PurchaseOrdersPage() {
  const { data, err, reload } = useList<PO>("/erp/purchase-orders", "orders");
  const suppliers = useList<Supplier>("/erp/suppliers", "suppliers");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<PO | null>(null);
  const [picked, setPicked] = useState<PO | null>(null);

  async function create(body: Partial<PO>) {
    await call(() => api.post<PO>("/erp/purchase-orders", { id: "", po_number: "", status: "DRAFT", ...body }));
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<PO>) {
    await call(() => api.patch<PO>(`/erp/purchase-orders/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this PO?")) return;
    await call(() => api.del<void>(`/erp/purchase-orders/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Purchase Orders"
        description="POs + lines → Goods Receipt → ledger RECEIPT."
        actions={<Button onClick={() => setOpen(true)}>New PO</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No purchase orders." />
      ) : (
        <Table head={["PO #", "Supplier", "Status", ""]}>
          {data.map((r) => (
            <tr key={r.id}>
              <Td className="font-mono text-xs">{r.po_number}</Td>
              <Td className="text-xs">{r.supplier_id?.slice(0, 8) ?? "—"}</Td>
              <Td>
                <Badge>{r.status}</Badge>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setPicked(r)}>
                  Lines
                </Button>
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <POForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        suppliers={suppliers.data ?? []}
        title="New purchase order"
      />
      <POForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        suppliers={suppliers.data ?? []}
        title={`Edit ${edit?.po_number ?? ""}`}
      />
      <POLinesPanel open={!!picked} order={picked} onClose={() => setPicked(null)} />
    </div>
  );
}

function POForm({
  open,
  initial,
  onClose,
  onSubmit,
  suppliers,
  title,
}: {
  open: boolean;
  initial?: PO;
  onClose: () => void;
  onSubmit: (b: Partial<PO>) => Promise<void> | void;
  suppliers: { id: string; name: string }[];
  title: string;
}) {
  const [supplierId, setSupplierId] = useState("");
  const [status, setStatus] = useState("DRAFT");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setSupplierId(initial?.supplier_id ?? "");
      setStatus(initial?.status ?? "DRAFT");
      setNotes(initial?.notes ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ supplier_id: supplierId || null, status, notes });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Supplier">
          <select
            value={supplierId}
            onChange={(e) => setSupplierId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            <option value="">— none —</option>
            {suppliers.map((s) => (
              <option key={s.id} value={s.id}>
                {s.name}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Status">
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {["DRAFT", "SUBMITTED", "APPROVED", "RECEIVED", "CANCELLED"].map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Notes">
          <Textarea rows={4} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

function POLinesPanel({ open, order, onClose }: { open: boolean; order: PO | null; onClose: () => void }) {
  const [lines, setLines] = useState<POLine[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const products = useList<Product>("/erp/products", "products");

  async function load() {
    if (!order) return;
    try {
      const r = await api.get<{ lines: POLine[] }>(`/erp/purchase-orders/${order.id}/lines`);
      setLines(r.lines);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => {
    if (open && order) void load();
  }, [open, order]);

  async function addLine(productId: string, qty: number, price: number) {
    if (!order) return;
    await call(() =>
      api.post<POLine>(`/erp/purchase-orders/${order.id}/lines`, {
        id: "",
        purchase_order_id: order.id,
        product_id: productId,
        quantity: qty,
        unit_price: price,
        notes: null,
      }),
    );
    await load();
  }
  async function del(id: string) {
    if (!order) return;
    if (!confirm("Delete line?")) return;
    await call(() => api.del<void>(`/erp/purchase-orders/${order.id}/lines/${id}`));
    await load();
  }

  return (
    <SidePanel open={open} onClose={onClose} title={`PO Lines — ${order?.po_number ?? ""}`} width={620}>
      {err && <Alert>{err}</Alert>}
      <div className="space-y-2">
        {!lines ? (
          <Spinner />
        ) : lines.length === 0 ? (
          <EmptyState message="No lines yet." />
        ) : (
          <Table head={["Product", "Qty", "Unit price", "Total", ""]}>
            {lines.map((l) => (
              <tr key={l.id}>
                <Td className="font-mono text-xs">{l.product_id.slice(0, 8)}</Td>
                <Td>{l.quantity}</Td>
                <Td>${l.unit_price.toFixed(2)}</Td>
                <Td>${(l.quantity * l.unit_price).toFixed(2)}</Td>
                <Td className="text-right">
                  <Button variant="danger" className="py-1 text-xs" onClick={() => del(l.id)}>
                    Remove
                  </Button>
                </Td>
              </tr>
            ))}
          </Table>
        )}
      </div>
      <div className="flex justify-end">
        <Button onClick={() => setAdding(true)}>Add line</Button>
      </div>
      <AddPOLinePanel open={adding} onClose={() => setAdding(false)} products={products.data ?? []} onAdd={addLine} />
    </SidePanel>
  );
}

function AddPOLinePanel({
  open,
  onClose,
  products,
  onAdd,
}: {
  open: boolean;
  onClose: () => void;
  products: { id: string; sku: string; name: string }[];
  onAdd: (productId: string, qty: number, price: number) => Promise<void> | void;
}) {
  const [productId, setProductId] = useState("");
  const [qty, setQty] = useState(1);
  const [price, setPrice] = useState(0);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setProductId(products[0]?.id ?? "");
      setQty(1);
      setPrice(0);
    }
  }, [open, products]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!productId) return;
    setBusy(true);
    try {
      await onAdd(productId, qty, price);
      onClose();
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="Add PO line" width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Product">
          <select
            required
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {products.map((p) => (
              <option key={p.id} value={p.id}>
                {p.sku} — {p.name}
              </option>
            ))}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Quantity">
            <Input type="number" min={0} step="any" value={qty} onChange={(e) => setQty(Number(e.target.value))} />
          </Field>
          <Field label="Unit price">
            <Input type="number" min={0} step="any" value={price} onChange={(e) => setPrice(Number(e.target.value))} />
          </Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Add
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Goods Receipts + Lines
// ============================================================================

type Receipt = {
  id: string;
  receipt_number: string;
  purchase_order_id?: string | null;
  warehouse_id?: string | null;
  received_at: string;
  notes?: string | null;
};
type ReceiptLine = {
  id: string;
  receipt_id: string;
  product_id: string;
  quantity: number;
  notes?: string | null;
};

export function GoodsReceiptsPage() {
  const { data, err, reload } = useList<Receipt>("/erp/goods-receipts", "receipts");
  const [open, setOpen] = useState(false);
  const [picked, setPicked] = useState<Receipt | null>(null);

  async function create(header: Partial<Receipt>, lines: Partial<ReceiptLine>[]) {
    await call(() => api.post<Receipt>("/erp/goods-receipts", { header, lines }));
    setOpen(false);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this receipt?")) return;
    await call(() => api.del<void>(`/erp/goods-receipts/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Goods Receipts"
        description="Receipt → ledger RECEIPT +qty per line."
        actions={<Button onClick={() => setOpen(true)}>New receipt</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No receipts." />
      ) : (
        <Table head={["Receipt #", "PO", "Warehouse", "Received", ""]}>
          {data.map((r) => (
            <tr key={r.id}>
              <Td className="font-mono text-xs">{r.receipt_number}</Td>
              <Td className="font-mono text-xs">{r.purchase_order_id?.slice(0, 8) ?? "—"}</Td>
              <Td className="font-mono text-xs">{r.warehouse_id?.slice(0, 8) ?? "—"}</Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">
                {new Date(r.received_at).toLocaleString()}
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setPicked(r)}>
                  Lines
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ReceiptForm open={open} onClose={() => setOpen(false)} onSubmit={create} />
      <ReceiptLinesPanel open={!!picked} receipt={picked} onClose={() => setPicked(null)} />
    </div>
  );
}

function ReceiptForm({
  open,
  onClose,
  onSubmit,
}: {
  open: boolean;
  onClose: () => void;
  onSubmit: (header: Partial<Receipt>, lines: Partial<ReceiptLine>[]) => Promise<void> | void;
}) {
  const pos = useList<PO>("/erp/purchase-orders", "orders");
  const warehouses = useList<Warehouse>("/erp/warehouses", "warehouses");
  const products = useList<Product>("/erp/products", "products");
  const [poId, setPoId] = useState("");
  const [warehouseId, setWarehouseId] = useState("");
  const [receivedAt, setReceivedAt] = useState(todayIso());
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<{ product_id: string; quantity: number }[]>([
    { product_id: "", quantity: 1 },
  ]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (open) {
      setPoId("");
      setWarehouseId(warehouses.data?.[0]?.id ?? "");
      setReceivedAt(todayIso());
      setNotes("");
      setLines([{ product_id: products.data?.[0]?.id ?? "", quantity: 1 }]);
    }
  }, [open, warehouses.data, products.data]);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      const header: Partial<Receipt> = {
        id: "",
        receipt_number: "",
        purchase_order_id: poId || null,
        warehouse_id: warehouseId || null,
        received_at: new Date(receivedAt).toISOString(),
        notes,
      };
      const ls: Partial<ReceiptLine>[] = lines
        .filter((l) => l.product_id)
        .map((l) => ({ id: "", receipt_id: "", product_id: l.product_id, quantity: l.quantity, notes: null }));
      await onSubmit(header, ls);
    } finally {
      setBusy(false);
    }
  }

  return (
    <SidePanel open={open} onClose={onClose} title="New goods receipt" width={620}>
      <form onSubmit={submit} className="space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <Field label="PO">
            <select
              value={poId}
              onChange={(e) => setPoId(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              <option value="">— none —</option>
              {(pos.data ?? []).map((p) => (
                <option key={p.id} value={p.id}>
                  {p.po_number}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Warehouse">
            <select
              required
              value={warehouseId}
              onChange={(e) => setWarehouseId(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              <option value="">— select —</option>
              {(warehouses.data ?? []).map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name}
                </option>
              ))}
            </select>
          </Field>
        </div>
        <Field label="Received at">
          <Input type="date" value={receivedAt} onChange={(e) => setReceivedAt(e.target.value)} />
        </Field>
        <Field label="Notes">
          <Textarea rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
        <div>
          <div className="mb-2 flex items-center justify-between">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
              Lines
            </h3>
            <Button
              type="button"
              variant="ghost"
              className="py-1 text-xs"
              onClick={() => setLines((ls) => [...ls, { product_id: products.data?.[0]?.id ?? "", quantity: 1 }])}
            >
              + Add line
            </Button>
          </div>
          <div className="space-y-2">
            {lines.map((l, i) => (
              <div key={i} className="grid grid-cols-[1fr_120px_36px] gap-2">
                <select
                  required
                  value={l.product_id}
                  onChange={(e) =>
                    setLines((ls) => ls.map((x, j) => (j === i ? { ...x, product_id: e.target.value } : x)))
                  }
                  className="rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
                >
                  <option value="">— product —</option>
                  {(products.data ?? []).map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.sku} — {p.name}
                    </option>
                  ))}
                </select>
                <Input
                  type="number"
                  min={0}
                  step="any"
                  value={l.quantity}
                  onChange={(e) =>
                    setLines((ls) => ls.map((x, j) => (j === i ? { ...x, quantity: Number(e.target.value) } : x)))
                  }
                />
                <button
                  type="button"
                  className="rounded-lg border border-[var(--app-border)] px-2 text-sm text-red-600 hover:bg-red-500/10"
                  onClick={() => setLines((ls) => ls.filter((_, j) => j !== i))}
                  disabled={lines.length === 1}
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

function ReceiptLinesPanel({ open, receipt, onClose }: { open: boolean; receipt: Receipt | null; onClose: () => void }) {
  const [lines, setLines] = useState<ReceiptLine[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const products = useList<Product>("/erp/products", "products");

  async function load() {
    if (!receipt) return;
    try {
      const r = await api.get<{ lines: ReceiptLine[] }>(`/erp/goods-receipts/${receipt.id}/lines`);
      setLines(r.lines);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => {
    if (open && receipt) void load();
  }, [open, receipt]);

  async function addLine(productId: string, qty: number) {
    if (!receipt) return;
    await call(() =>
      api.post<ReceiptLine>(`/erp/goods-receipts/${receipt.id}/lines`, {
        id: "",
        receipt_id: receipt.id,
        product_id: productId,
        quantity: qty,
        notes: null,
      }),
    );
    await load();
  }
  async function del(id: string) {
    if (!receipt) return;
    if (!confirm("Delete line?")) return;
    await call(() => api.del<void>(`/erp/goods-receipts/${receipt.id}/lines/${id}`));
    await load();
  }

  return (
    <SidePanel open={open} onClose={onClose} title={`Receipt Lines — ${receipt?.receipt_number ?? ""}`} width={520}>
      {err && <Alert>{err}</Alert>}
      <div className="space-y-2">
        {!lines ? (
          <Spinner />
        ) : lines.length === 0 ? (
          <EmptyState message="No lines yet." />
        ) : (
          <Table head={["Product", "Qty", ""]}>
            {lines.map((l) => (
              <tr key={l.id}>
                <Td className="font-mono text-xs">{l.product_id.slice(0, 8)}</Td>
                <Td>{l.quantity}</Td>
                <Td className="text-right">
                  <Button variant="danger" className="py-1 text-xs" onClick={() => del(l.id)}>
                    Remove
                  </Button>
                </Td>
              </tr>
            ))}
          </Table>
        )}
      </div>
      <div className="flex justify-end">
        <Button onClick={() => setAdding(true)}>Add line</Button>
      </div>
      <AddReceiptLinePanel
        open={adding}
        onClose={() => setAdding(false)}
        products={products.data ?? []}
        onAdd={addLine}
      />
    </SidePanel>
  );
}

function AddReceiptLinePanel({
  open,
  onClose,
  products,
  onAdd,
}: {
  open: boolean;
  onClose: () => void;
  products: { id: string; sku: string; name: string }[];
  onAdd: (productId: string, qty: number) => Promise<void> | void;
}) {
  const [productId, setProductId] = useState("");
  const [qty, setQty] = useState(1);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setProductId(products[0]?.id ?? "");
      setQty(1);
    }
  }, [open, products]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!productId) return;
    setBusy(true);
    try {
      await onAdd(productId, qty);
      onClose();
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="Add receipt line" width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Product">
          <select
            required
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {products.map((p) => (
              <option key={p.id} value={p.id}>
                {p.sku} — {p.name}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Quantity">
          <Input type="number" min={0} step="any" value={qty} onChange={(e) => setQty(Number(e.target.value))} />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Add
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Inventory — transactions + stock
// ============================================================================

type InventoryTx = {
  id: string;
  product_id: string;
  warehouse_id: string;
  transaction_type: string;
  quantity: number;
  reference_type?: string | null;
  reference_id?: string | null;
};
type StockLevel = { product_id: string; warehouse_id: string; quantity: number };

export function InventoryPage() {
  const { data: txs, err } = useList<InventoryTx>("/erp/inventory/transactions", "transactions");
  const [stock, setStock] = useState<StockLevel[] | null>(null);
  useEffect(() => {
    api
      .get<{ stock: StockLevel[] }>("/erp/inventory/stock")
      .then((r) => setStock(r.stock))
      .catch(() => setStock([]));
  }, []);

  return (
    <div className="space-y-6">
      <PageHeader title="Inventory" description="Stock transaction ledger — Stock = Σ quantity per (product, warehouse)." />
      {err && <Alert>{err}</Alert>}
      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            Current stock
          </h3>
          <div className="mt-3">
            {!stock ? (
              <Spinner />
            ) : stock.length === 0 ? (
              <EmptyState message="No ledger entries." />
            ) : (
              <Table head={["Product", "Warehouse", "Qty"]}>
                {stock.map((s) => (
                  <tr key={`${s.product_id}-${s.warehouse_id}`}>
                    <Td className="font-mono text-xs">{s.product_id.slice(0, 8)}</Td>
                    <Td className="font-mono text-xs">{s.warehouse_id.slice(0, 8)}</Td>
                    <Td className={s.quantity < 0 ? "text-red-600" : "text-emerald-600"}>
                      {s.quantity}
                    </Td>
                  </tr>
                ))}
              </Table>
            )}
          </div>
        </Card>
        <Card className="p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            Ledger (last 200)
          </h3>
          <div className="mt-3">
            {!txs ? (
              <Spinner />
            ) : txs.length === 0 ? (
              <EmptyState message="No transactions." />
            ) : (
              <Table head={["Type", "Qty", "Ref"]}>
                {txs.slice(0, 12).map((t) => (
                  <tr key={t.id}>
                    <Td>
                      <Badge tone={t.quantity > 0 ? "green" : "red"}>{t.transaction_type}</Badge>
                    </Td>
                    <Td>{t.quantity}</Td>
                    <Td className="font-mono text-xs">
                      {t.reference_type ? `${t.reference_type}:${t.reference_id?.slice(0, 8) ?? ""}` : "—"}
                    </Td>
                  </tr>
                ))}
              </Table>
            )}
          </div>
        </Card>
      </div>
    </div>
  );
}

// ============================================================================
// Sales Orders + Lines
// ============================================================================

type SO = {
  id: string;
  so_number: string;
  customer_name: string;
  status: string;
  notes?: string | null;
};
type SOLine = {
  id: string;
  sales_order_id: string;
  product_id: string;
  quantity: number;
  unit_price: number;
  notes?: string | null;
};

export function SalesOrdersPage() {
  const { data, err, reload } = useList<SO>("/erp/sales-orders", "orders");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<SO | null>(null);
  const [picked, setPicked] = useState<SO | null>(null);

  async function create(body: Partial<SO>) {
    await call(() =>
      api.post<SO>("/erp/sales-orders", { id: "", so_number: "", status: "DRAFT", ...body }),
    );
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<SO>) {
    await call(() => api.patch<SO>(`/erp/sales-orders/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this SO?")) return;
    await call(() => api.del<void>(`/erp/sales-orders/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Sales Orders"
        description="SO + lines → Shipment → ledger Shipment (−)."
        actions={<Button onClick={() => setOpen(true)}>New SO</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No sales orders." />
      ) : (
        <Table head={["SO #", "Customer", "Status", ""]}>
          {data.map((r) => (
            <tr key={r.id}>
              <Td className="font-mono text-xs">{r.so_number}</Td>
              <Td>{r.customer_name}</Td>
              <Td>
                <Badge>{r.status}</Badge>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setPicked(r)}>
                  Lines
                </Button>
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <SOForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New sales order"
      />
      <SOForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit ${edit?.so_number ?? ""}`}
      />
      <SOLinesPanel open={!!picked} order={picked} onClose={() => setPicked(null)} />
    </div>
  );
}

function SOForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: SO;
  onClose: () => void;
  onSubmit: (b: Partial<SO>) => Promise<void> | void;
  title: string;
}) {
  const [customer, setCustomer] = useState("");
  const [status, setStatus] = useState("DRAFT");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setCustomer(initial?.customer_name ?? "");
      setStatus(initial?.status ?? "DRAFT");
      setNotes(initial?.notes ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ customer_name: customer, status, notes });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Customer">
          <Input required value={customer} onChange={(e) => setCustomer(e.target.value)} />
        </Field>
        <Field label="Status">
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {["DRAFT", "CONFIRMED", "SHIPPED", "DELIVERED", "CANCELLED"].map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Notes">
          <Textarea rows={4} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

function SOLinesPanel({ open, order, onClose }: { open: boolean; order: SO | null; onClose: () => void }) {
  const [lines, setLines] = useState<SOLine[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [adding, setAdding] = useState(false);
  const products = useList<Product>("/erp/products", "products");

  async function load() {
    if (!order) return;
    try {
      const r = await api.get<{ lines: SOLine[] }>(`/erp/sales-orders/${order.id}/lines`);
      setLines(r.lines);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => {
    if (open && order) void load();
  }, [open, order]);

  async function addLine(productId: string, qty: number, price: number) {
    if (!order) return;
    await call(() =>
      api.post<SOLine>(`/erp/sales-orders/${order.id}/lines`, {
        id: "",
        sales_order_id: order.id,
        product_id: productId,
        quantity: qty,
        unit_price: price,
        notes: null,
      }),
    );
    await load();
  }
  async function del(id: string) {
    if (!order) return;
    if (!confirm("Delete line?")) return;
    await call(() => api.del<void>(`/erp/sales-orders/${order.id}/lines/${id}`));
    await load();
  }

  return (
    <SidePanel open={open} onClose={onClose} title={`SO Lines — ${order?.so_number ?? ""}`} width={620}>
      {err && <Alert>{err}</Alert>}
      <div className="space-y-2">
        {!lines ? (
          <Spinner />
        ) : lines.length === 0 ? (
          <EmptyState message="No lines yet." />
        ) : (
          <Table head={["Product", "Qty", "Unit price", "Total", ""]}>
            {lines.map((l) => (
              <tr key={l.id}>
                <Td className="font-mono text-xs">{l.product_id.slice(0, 8)}</Td>
                <Td>{l.quantity}</Td>
                <Td>${l.unit_price.toFixed(2)}</Td>
                <Td>${(l.quantity * l.unit_price).toFixed(2)}</Td>
                <Td className="text-right">
                  <Button variant="danger" className="py-1 text-xs" onClick={() => del(l.id)}>
                    Remove
                  </Button>
                </Td>
              </tr>
            ))}
          </Table>
        )}
      </div>
      <div className="flex justify-end">
        <Button onClick={() => setAdding(true)}>Add line</Button>
      </div>
      <AddSOLinePanel open={adding} onClose={() => setAdding(false)} products={products.data ?? []} onAdd={addLine} />
    </SidePanel>
  );
}

function AddSOLinePanel({
  open,
  onClose,
  products,
  onAdd,
}: {
  open: boolean;
  onClose: () => void;
  products: { id: string; sku: string; name: string }[];
  onAdd: (productId: string, qty: number, price: number) => Promise<void> | void;
}) {
  const [productId, setProductId] = useState("");
  const [qty, setQty] = useState(1);
  const [price, setPrice] = useState(0);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setProductId(products[0]?.id ?? "");
      setQty(1);
      setPrice(0);
    }
  }, [open, products]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!productId) return;
    setBusy(true);
    try {
      await onAdd(productId, qty, price);
      onClose();
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="Add SO line" width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Product">
          <select
            required
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {products.map((p) => (
              <option key={p.id} value={p.id}>
                {p.sku} — {p.name}
              </option>
            ))}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Quantity">
            <Input type="number" min={0} step="any" value={qty} onChange={(e) => setQty(Number(e.target.value))} />
          </Field>
          <Field label="Unit price">
            <Input type="number" min={0} step="any" value={price} onChange={(e) => setPrice(Number(e.target.value))} />
          </Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Add
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Shipments
// ============================================================================

type Shipment = {
  id: string;
  sales_order_id: string;
  warehouse_id?: string | null;
  shipped_at: string;
  notes?: string | null;
};

export function ShipmentsPage() {
  const { data, err, reload } = useList<Shipment>("/erp/shipments", "shipments");
  const sos = useList<SO>("/erp/sales-orders", "orders");
  const warehouses = useList<Warehouse>("/erp/warehouses", "warehouses");
  const products = useList<Product>("/erp/products", "products");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Shipment | null>(null);

  async function create(header: Partial<Shipment>, lines: { product_id: string; quantity: number }[]) {
    await call(() => {
      const v: any = {
        id: "",
        sales_order_id: header.sales_order_id,
        warehouse_id: header.warehouse_id ?? null,
        shipped_at: header.shipped_at,
        notes: header.notes ?? null,
      };
      const ls = lines.map((l) => ({
        id: "",
        sales_order_id: header.sales_order_id,
        product_id: l.product_id,
        quantity: l.quantity,
        unit_price: 0,
        notes: null,
      }));
      return api.post<Shipment>("/erp/shipments", { header: v, lines: ls });
    });
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Shipment>) {
    await call(() => api.patch<Shipment>(`/erp/shipments/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this shipment?")) return;
    await call(() => api.del<void>(`/erp/shipments/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Shipments"
        description="Outbound shipments → ledger Shipment (−qty per line)."
        actions={<Button onClick={() => setOpen(true)}>New shipment</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No shipments." />
      ) : (
        <Table head={["SO", "Warehouse", "Shipped at", ""]}>
          {data.map((s) => (
            <tr key={s.id}>
              <Td className="font-mono text-xs">{s.sales_order_id.slice(0, 8)}</Td>
              <Td className="font-mono text-xs">{s.warehouse_id?.slice(0, 8) ?? "—"}</Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">
                {new Date(s.shipped_at).toLocaleString()}
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(s)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(s.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ShipmentForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        sos={sos.data ?? []}
        warehouses={warehouses.data ?? []}
        products={products.data ?? []}
        title="New shipment"
      />
      <ShipmentForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (h) => { if (edit) await update(edit.id, h); }}
        sos={sos.data ?? []}
        warehouses={warehouses.data ?? []}
        products={products.data ?? []}
        title={`Edit shipment ${edit?.id.slice(0, 8) ?? ""}`}
      />
    </div>
  );
}

function ShipmentForm({
  open,
  initial,
  onClose,
  onSubmit,
  sos,
  warehouses,
  products,
  title,
}: {
  open: boolean;
  initial?: Shipment;
  onClose: () => void;
  onSubmit: (h: Partial<Shipment>, l: { product_id: string; quantity: number }[]) => Promise<void> | void;
  sos: { id: string; so_number: string }[];
  warehouses: { id: string; name: string }[];
  products: { id: string; sku: string; name: string }[];
  title: string;
}) {
  const [soId, setSoId] = useState("");
  const [warehouseId, setWarehouseId] = useState("");
  const [shippedAt, setShippedAt] = useState(todayIso());
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<{ product_id: string; quantity: number }[]>([
    { product_id: "", quantity: 1 },
  ]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (open) {
      setSoId(initial?.sales_order_id ?? sos[0]?.id ?? "");
      setWarehouseId(initial?.warehouse_id ?? warehouses[0]?.id ?? "");
      setShippedAt(initial?.shipped_at ? initial.shipped_at.slice(0, 10) : todayIso());
      setNotes(initial?.notes ?? "");
      setLines([{ product_id: products[0]?.id ?? "", quantity: 1 }]);
    }
  }, [open, initial, sos, warehouses, products]);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit(
        {
          sales_order_id: soId,
          warehouse_id: warehouseId || null,
          shipped_at: new Date(shippedAt).toISOString(),
          notes,
        },
        lines.filter((l) => l.product_id),
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <SidePanel open={open} onClose={onClose} title={title} width={620}>
      <form onSubmit={submit} className="space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <Field label="Sales order">
            <select
              required
              value={soId}
              onChange={(e) => setSoId(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              {sos.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.so_number}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Warehouse">
            <select
              value={warehouseId}
              onChange={(e) => setWarehouseId(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              <option value="">— none —</option>
              {warehouses.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name}
                </option>
              ))}
            </select>
          </Field>
        </div>
        <Field label="Shipped at">
          <Input type="date" value={shippedAt} onChange={(e) => setShippedAt(e.target.value)} />
        </Field>
        <Field label="Notes">
          <Textarea rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
        <div>
          <div className="mb-2 flex items-center justify-between">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
              Lines
            </h3>
            <Button
              type="button"
              variant="ghost"
              className="py-1 text-xs"
              onClick={() => setLines((ls) => [...ls, { product_id: products[0]?.id ?? "", quantity: 1 }])}
            >
              + Add line
            </Button>
          </div>
          <div className="space-y-2">
            {lines.map((l, i) => (
              <div key={i} className="grid grid-cols-[1fr_120px_36px] gap-2">
                <select
                  required
                  value={l.product_id}
                  onChange={(e) =>
                    setLines((ls) => ls.map((x, j) => (j === i ? { ...x, product_id: e.target.value } : x)))
                  }
                  className="rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
                >
                  <option value="">— product —</option>
                  {products.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.sku} — {p.name}
                    </option>
                  ))}
                </select>
                <Input
                  type="number"
                  min={0}
                  step="any"
                  value={l.quantity}
                  onChange={(e) =>
                    setLines((ls) => ls.map((x, j) => (j === i ? { ...x, quantity: Number(e.target.value) } : x)))
                  }
                />
                <button
                  type="button"
                  className="rounded-lg border border-[var(--app-border)] px-2 text-sm text-red-600 hover:bg-red-500/10"
                  onClick={() => setLines((ls) => ls.filter((_, j) => j !== i))}
                  disabled={lines.length === 1}
                >
                  ×
                </button>
              </div>
            ))}
          </div>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Assets
// ============================================================================

type Asset = {
  id: string;
  asset_number: string;
  product_id?: string | null;
  name: string;
  acquisition_cost: number;
  acquisition_date: string;
  status: string;
  location_id?: string | null;
};

export function AssetsPage() {
  const { data, err, reload } = useList<Asset>("/erp/assets", "assets");
  const products = useList<Product>("/erp/products", "products");
  const warehouses = useList<Warehouse>("/erp/warehouses", "warehouses");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Asset | null>(null);

  async function create(body: Partial<Asset>) {
    await call(() =>
      api.post<Asset>("/erp/assets", {
        id: "",
        asset_number: "",
        status: "active",
        ...body,
      }),
    );
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Asset>) {
    await call(() => api.patch<Asset>(`/erp/assets/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this asset?")) return;
    await call(() => api.del<void>(`/erp/assets/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Asset Management"
        description="Purchase → Goods Receipt → Asset Capitalization."
        actions={<Button onClick={() => setOpen(true)}>New asset</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No assets." />
      ) : (
        <Table head={["Asset #", "Name", "Cost", "Status", ""]}>
          {data.map((r) => (
            <tr key={r.id}>
              <Td className="font-mono text-xs">{r.asset_number}</Td>
              <Td className="font-medium">{r.name}</Td>
              <Td>${r.acquisition_cost.toFixed(2)}</Td>
              <Td>
                <Badge>{r.status}</Badge>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <AssetForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        products={products.data ?? []}
        warehouses={warehouses.data ?? []}
        title="New asset"
      />
      <AssetForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        products={products.data ?? []}
        warehouses={warehouses.data ?? []}
        title={`Edit ${edit?.asset_number ?? ""}`}
      />
    </div>
  );
}

function AssetForm({
  open,
  initial,
  onClose,
  onSubmit,
  products,
  warehouses,
  title,
}: {
  open: boolean;
  initial?: Asset;
  onClose: () => void;
  onSubmit: (b: Partial<Asset>) => Promise<void> | void;
  products: { id: string; sku: string; name: string }[];
  warehouses: { id: string; name: string }[];
  title: string;
}) {
  const [name, setName] = useState("");
  const [productId, setProductId] = useState("");
  const [cost, setCost] = useState(0);
  const [date, setDate] = useState(todayIso());
  const [status, setStatus] = useState("active");
  const [locationId, setLocationId] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setName(initial?.name ?? "");
      setProductId(initial?.product_id ?? "");
      setCost(initial?.acquisition_cost ?? 0);
      setDate(initial?.acquisition_date ? initial.acquisition_date.slice(0, 10) : todayIso());
      setStatus(initial?.status ?? "active");
      setLocationId(initial?.location_id ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({
        name,
        product_id: productId || null,
        acquisition_cost: cost,
        acquisition_date: new Date(date).toISOString(),
        status,
        location_id: locationId || null,
      });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={460}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name">
          <Input required value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        <Field label="Product">
          <select
            value={productId}
            onChange={(e) => setProductId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            <option value="">— none —</option>
            {products.map((p) => (
              <option key={p.id} value={p.id}>
                {p.sku} — {p.name}
              </option>
            ))}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Cost">
            <Input type="number" min={0} step="any" value={cost} onChange={(e) => setCost(Number(e.target.value))} />
          </Field>
          <Field label="Date">
            <Input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
          </Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Status">
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              {["active", "disposed", "sold", "under_repair"].map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Location">
            <select
              value={locationId}
              onChange={(e) => setLocationId(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              <option value="">— none —</option>
              {warehouses.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name}
                </option>
              ))}
            </select>
          </Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Expenses
// ============================================================================

type Expense = {
  id: string;
  expense_number: string;
  category: string;
  description: string;
  amount: number;
  expense_date: string;
  status: string;
  approved_by?: string | null;
  notes?: string | null;
};

const EXPENSE_CATEGORIES = ["Travel", "Meals", "Office", "Marketing", "Software", "Logistics", "Other"];

export function ExpensesPage() {
  const { data, err, reload } = useList<Expense>("/erp/expenses", "expenses");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Expense | null>(null);

  async function create(body: Partial<Expense>) {
    await call(() =>
      api.post<Expense>("/erp/expenses", {
        id: "",
        expense_number: "",
        status: "DRAFT",
        ...body,
      }),
    );
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Expense>) {
    await call(() => api.patch<Expense>(`/erp/expenses/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this expense?")) return;
    await call(() => api.del<void>(`/erp/expenses/${id}`));
    await reload();
  }
  async function clearAll() {
    if (!confirm("Delete ALL expenses? This cannot be undone.")) return;
    await call(() => api.del<void>("/erp/expenses/clear"));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Expense Management"
        description="Standalone: Expense → Approval → Payment."
        actions={
          <div className="flex gap-2">
            <Button variant="danger" onClick={clearAll}>
              Clear all
            </Button>
            <Button onClick={() => setOpen(true)}>New expense</Button>
          </div>
        }
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No expenses." />
      ) : (
        <Table head={["Expense #", "Category", "Description", "Amount", "Date", "Status", ""]}>
          {data.map((r) => (
            <tr key={r.id}>
              <Td className="font-mono text-xs">{r.expense_number}</Td>
              <Td>
                <Badge>{r.category}</Badge>
              </Td>
              <Td className="max-w-[260px] truncate text-sm">{r.description}</Td>
              <Td>${r.amount.toFixed(2)}</Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">
                {new Date(r.expense_date).toLocaleDateString()}
              </Td>
              <Td>
                <Badge>{r.status}</Badge>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ExpenseForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New expense"
      />
      <ExpenseForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit ${edit?.expense_number ?? ""}`}
      />
    </div>
  );
}

function ExpenseForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: Expense;
  onClose: () => void;
  onSubmit: (b: Partial<Expense>) => Promise<void> | void;
  title: string;
}) {
  const [category, setCategory] = useState("Other");
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState(0);
  const [date, setDate] = useState(todayIso());
  const [status, setStatus] = useState("DRAFT");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setCategory(initial?.category ?? "Other");
      setDescription(initial?.description ?? "");
      setAmount(initial?.amount ?? 0);
      setDate(initial?.expense_date ? initial.expense_date.slice(0, 10) : todayIso());
      setStatus(initial?.status ?? "DRAFT");
      setNotes(initial?.notes ?? "");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({
        category,
        description,
        amount,
        expense_date: new Date(date).toISOString(),
        status,
        notes,
      });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={460}>
      <form onSubmit={submit} className="space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <Field label="Category">
            <select
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              {EXPENSE_CATEGORIES.map((c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Status">
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
            >
              {["DRAFT", "SUBMITTED", "APPROVED", "PAID", "REJECTED"].map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </Field>
        </div>
        <Field label="Description">
          <Input required value={description} onChange={(e) => setDescription(e.target.value)} />
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Amount">
            <Input type="number" min={0} step="any" value={amount} onChange={(e) => setAmount(Number(e.target.value))} />
          </Field>
          <Field label="Date">
            <Input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
          </Field>
        </div>
        <Field label="Notes">
          <Textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} />
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Invoices (accounting)
// ============================================================================

type Invoice = {
  id: string;
  invoice_number: string;
  sales_order_id?: string | null;
  quote_id?: string | null;
  total: number;
  status: string;
};

export function InvoicesPage() {
  const { data, err, reload } = useList<Invoice>("/accounting/invoices", "invoices");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Invoice | null>(null);

  async function create(body: Partial<Invoice>) {
    await call(() =>
      api.post<Invoice>("/accounting/invoices", {
        id: "",
        invoice_number: "",
        status: "DRAFT",
        total: 0,
        ...body,
      }),
    );
    setOpen(false);
    await reload();
  }
  async function update(id: string, body: Partial<Invoice>) {
    await call(() => api.patch<Invoice>(`/accounting/invoices/${id}`, body));
    setEdit(null);
    await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this invoice?")) return;
    await call(() => api.del<void>(`/accounting/invoices/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Invoices"
        description="Auto-created on Quote Accept (sales order → invoice draft). Manual entry also supported."
        actions={<Button onClick={() => setOpen(true)}>New invoice</Button>}
      />
      {err && <Alert>{err}</Alert>}
      {!data ? (
        <Spinner />
      ) : data.length === 0 ? (
        <EmptyState message="No invoices." />
      ) : (
        <Table head={["Invoice #", "Total", "Status", ""]}>
          {data.map((r) => (
            <tr key={r.id}>
              <Td className="font-mono text-xs">{r.invoice_number}</Td>
              <Td>${r.total.toFixed(2)}</Td>
              <Td>
                <Badge>{r.status}</Badge>
              </Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>
                  Edit
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>
                  Delete
                </Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <InvoiceForm
        open={open}
        onClose={() => setOpen(false)}
        onSubmit={create}
        title="New invoice"
      />
      <InvoiceForm
        open={!!edit}
        initial={edit ?? undefined}
        onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }}
        title={`Edit ${edit?.invoice_number ?? ""}`}
      />
    </div>
  );
}

function InvoiceForm({
  open,
  initial,
  onClose,
  onSubmit,
  title,
}: {
  open: boolean;
  initial?: Invoice;
  onClose: () => void;
  onSubmit: (b: Partial<Invoice>) => Promise<void> | void;
  title: string;
}) {
  const [total, setTotal] = useState(0);
  const [status, setStatus] = useState("DRAFT");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) {
      setTotal(initial?.total ?? 0);
      setStatus(initial?.status ?? "DRAFT");
    }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    try {
      await onSubmit({ total, status });
    } finally {
      setBusy(false);
    }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Total">
          <Input
            type="number"
            min={0}
            step="any"
            value={total}
            onChange={(e) => setTotal(Number(e.target.value))}
          />
        </Field>
        <Field label="Status">
          <select
            value={status}
            onChange={(e) => setStatus(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          >
            {["DRAFT", "SENT", "PAID", "OVERDUE", "CANCELLED"].map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </form>
    </SidePanel>
  );
}

export default ErpLayout;
