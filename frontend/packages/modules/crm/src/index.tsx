// CRM module — Lead → Opportunity → Customer → Contact → Activity → Quote
// Full CRUD with SidePanel forms, edit/delete per record.
import { useEffect, useState } from "react";
import { NavLink, Outlet, useLocation, useNavigate } from "react-router";
import { api } from "@gateway/lib";
import {
  Alert, Badge, Button, Card, EmptyState, Field, Input, PageHeader,
  SidePanel, Spinner, Table, Td, Textarea,
} from "@gateway/ui";

const cn = (...p: Array<string | false | null | undefined>) => p.filter(Boolean).join(" ");

type SubNav = { to: string; label: string; desc: string };
const CRM_NAV: SubNav[] = [
  { to: "/crm/leads", label: "Leads", desc: "Inbound prospects" },
  { to: "/crm/opportunities", label: "Opportunities", desc: "Pipeline" },
  { to: "/crm/customers", label: "Customers", desc: "Accounts" },
  { to: "/crm/contacts", label: "Contacts", desc: "People" },
  { to: "/crm/activities", label: "Activities", desc: "Tasks/calls" },
  { to: "/crm/quotes", label: "Quotes", desc: "→ Sales" },
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

export function CrmLayout() {
  const loc = useLocation();
  return (
    <div className="flex gap-6">
      <aside className="hidden w-[210px] shrink-0 lg:block"
        style={{ position: "sticky", top: 80, alignSelf: "flex-start", height: "calc(100dvh - 96px)" }}>
        <div className="rounded-xl border border-[var(--app-border)] bg-[var(--app-card)] p-3">
          <div className="px-2 pb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">CRM</div>
          <nav className="flex flex-col gap-0.5">
            {CRM_NAV.map((i) => (
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
          {CRM_NAV.map((i) => (
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
        {loc.pathname === "/crm" && <CrmOverview />}
      </div>
    </div>
  );
}

export function CrmOverview() {
  const nav = useNavigate();
  return (
    <div className="space-y-6">
      <PageHeader title="CRM" description="Leads → Opportunities → Customers → Quotes → Sales" />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {CRM_NAV.map((i) => (
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

// ================================================================== Leads ===

type Lead = { id: string; lead_number: string; company_name?: string | null; contact_name: string; email?: string | null; phone?: string | null; source?: string | null; status: string; estimated_value?: number | null; notes?: string | null };

export function LeadsPage() {
  const { data, err, reload } = useList<Lead>("/crm/leads", "leads");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Lead | null>(null);

  async function create(body: Partial<Lead>) {
    await call(() => api.post<Lead>("/crm/leads", { id: "", lead_number: "", contact_name: "", status: "NEW", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Lead>) {
    await call(() => api.patch<Lead>(`/crm/leads/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this lead?")) return;
    await call(() => api.del<void>(`/crm/leads/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Leads" description="Lead → Opportunity → Customer → Quote → Sales."
        actions={<Button onClick={() => setOpen(true)}>New lead</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No leads yet." /> : (
        <Table head={["Lead #", "Company", "Contact", "Status", "Value", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{r.lead_number}</Td>
              <Td>{r.company_name ?? "—"}</Td>
              <Td className="font-medium">{r.contact_name}</Td>
              <Td><Badge>{r.status}</Badge></Td>
              <Td>{r.estimated_value != null ? `$${r.estimated_value.toFixed(0)}` : "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <LeadForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New lead" />
      <LeadForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.lead_number ?? ""}`} />
    </div>
  );
}

function LeadForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Lead; onClose: () => void;
  onSubmit: (b: Partial<Lead>) => Promise<void> | void; title: string;
}) {
  const [company, setCompany] = useState(""); const [contact, setContact] = useState("");
  const [email, setEmail] = useState(""); const [phone, setPhone] = useState("");
  const [source, setSource] = useState("web"); const [status, setStatus] = useState("NEW");
  const [value, setValue] = useState(""); const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setCompany(initial?.company_name ?? ""); setContact(initial?.contact_name ?? "");
      setEmail(initial?.email ?? ""); setPhone(initial?.phone ?? ""); setSource(initial?.source ?? "web");
      setStatus(initial?.status ?? "NEW"); setValue(initial?.estimated_value?.toString() ?? ""); setNotes(initial?.notes ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ company_name: company || null, contact_name: contact, email: email || null,
      phone: phone || null, source: source || null, status, estimated_value: value ? parseFloat(value) : null, notes: notes || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={440}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Company"><Input value={company} onChange={(e) => setCompany(e.target.value)} /></Field>
        <Field label="Contact Name"><Input required value={contact} onChange={(e) => setContact(e.target.value)} /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Email"><Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} /></Field>
          <Field label="Phone"><Input value={phone} onChange={(e) => setPhone(e.target.value)} /></Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Source">
            <select value={source} onChange={(e) => setSource(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["web", "referral", "cold_call", "event", "other"].map((s) => <option key={s} value={s}>{s}</option>)}
            </select>
          </Field>
          <Field label="Status">
            <select value={status} onChange={(e) => setStatus(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["NEW", "CONTACTED", "QUALIFIED", "UNQUALIFIED"].map((s) => <option key={s} value={s}>{s}</option>)}
            </select>
          </Field>
        </div>
        <Field label="Estimated Value"><Input type="number" min={0} step="any" value={value} onChange={(e) => setValue(e.target.value)} /></Field>
        <Field label="Notes"><Textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// =========================================================== Opportunities ===

type Opportunity = { id: string; opp_number: string; stage: string; amount?: number | null; probability?: number | null; notes?: string | null };

export function OpportunitiesPage() {
  const { data, err, reload } = useList<Opportunity>("/crm/opportunities", "opportunities");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Opportunity | null>(null);

  async function create(body: Partial<Opportunity>) {
    await call(() => api.post<Opportunity>("/crm/opportunities", { id: "", opp_number: "", stage: "PROSPECT", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Opportunity>) {
    await call(() => api.patch<Opportunity>(`/crm/opportunities/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this opportunity?")) return;
    await call(() => api.del<void>(`/crm/opportunities/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Opportunities" description="PROSPECT → QUALIFIED → PROPOSAL → NEGOTIATION → WON/LOST"
        actions={<Button onClick={() => setOpen(true)}>New opportunity</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No opportunities." /> : (
        <Table head={["Opp #", "Stage", "Amount", "Probability", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{r.opp_number}</Td>
              <Td><Badge>{r.stage}</Badge></Td>
              <Td>{r.amount != null ? `$${r.amount.toFixed(0)}` : "—"}</Td>
              <Td>{r.probability != null ? `${r.probability}%` : "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <OppForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New opportunity" />
      <OppForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.opp_number ?? ""}`} />
    </div>
  );
}

function OppForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Opportunity; onClose: () => void;
  onSubmit: (b: Partial<Opportunity>) => Promise<void> | void; title: string;
}) {
  const [stage, setStage] = useState("PROSPECT"); const [amount, setAmount] = useState("");
  const [prob, setProb] = useState(""); const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setStage(initial?.stage ?? "PROSPECT"); setAmount(initial?.amount?.toString() ?? "");
      setProb(initial?.probability?.toString() ?? ""); setNotes(initial?.notes ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ stage, amount: amount ? parseFloat(amount) : null,
      probability: prob ? parseFloat(prob) : null, notes: notes || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Stage">
          <select value={stage} onChange={(e) => setStage(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {["PROSPECT", "QUALIFIED", "PROPOSAL", "NEGOTIATION", "WON", "LOST"].map((s) => <option key={s} value={s}>{s}</option>)}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Amount"><Input type="number" min={0} step="any" value={amount} onChange={(e) => setAmount(e.target.value)} /></Field>
          <Field label="Probability %"><Input type="number" min={0} max={100} value={prob} onChange={(e) => setProb(e.target.value)} /></Field>
        </div>
        <Field label="Notes"><Textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================ Customers ===

type Customer = { id: string; customer_number: string; name: string; email?: string | null; phone?: string | null; address?: string | null };

export function CustomersPage() {
  const { data, err, reload } = useList<Customer>("/crm/customers", "customers");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Customer | null>(null);

  async function create(body: Partial<Customer>) {
    await call(() => api.post<Customer>("/crm/customers", { id: "", customer_number: "", name: "", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Customer>) {
    await call(() => api.patch<Customer>(`/crm/customers/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this customer?")) return;
    await call(() => api.del<void>(`/crm/customers/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Customers" description="Contact 360 — feeds Quote → Sales."
        actions={<Button onClick={() => setOpen(true)}>New customer</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No customers." /> : (
        <Table head={["Customer #", "Name", "Email", "Phone", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{r.customer_number}</Td>
              <Td className="font-medium">{r.name}</Td>
              <Td className="text-xs">{r.email ?? "—"}</Td>
              <Td className="text-xs">{r.phone ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <CustomerForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New customer" />
      <CustomerForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function CustomerForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Customer; onClose: () => void;
  onSubmit: (b: Partial<Customer>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [email, setEmail] = useState("");
  const [phone, setPhone] = useState(""); const [address, setAddress] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setEmail(initial?.email ?? "");
      setPhone(initial?.phone ?? ""); setAddress(initial?.address ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, email: email || null, phone: phone || null, address: address || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Email"><Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} /></Field>
          <Field label="Phone"><Input value={phone} onChange={(e) => setPhone(e.target.value)} /></Field>
        </div>
        <Field label="Address"><Input value={address} onChange={(e) => setAddress(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================== Contacts ===

type Contact = { id: string; name: string; role?: string | null; email?: string | null; phone?: string | null; customer_id?: string | null };

export function ContactsPage() {
  const { data, err, reload } = useList<Contact>("/crm/contacts", "contacts");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Contact | null>(null);

  async function create(body: Partial<Contact>) {
    await call(() => api.post<Contact>("/crm/contacts", { id: "", name: "", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Contact>) {
    await call(() => api.patch<Contact>(`/crm/contacts/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this contact?")) return;
    await call(() => api.del<void>(`/crm/contacts/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Contacts" description="Sub-contacts of a Customer."
        actions={<Button onClick={() => setOpen(true)}>New contact</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No contacts." /> : (
        <Table head={["Name", "Role", "Email", "Phone", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td><Badge>{r.role ?? "—"}</Badge></Td>
              <Td className="text-xs">{r.email ?? "—"}</Td>
              <Td className="text-xs">{r.phone ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ContactForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New contact" />
      <ContactForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function ContactForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Contact; onClose: () => void;
  onSubmit: (b: Partial<Contact>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [role, setRole] = useState("");
  const [email, setEmail] = useState(""); const [phone, setPhone] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setRole(initial?.role ?? "");
      setEmail(initial?.email ?? ""); setPhone(initial?.phone ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, role: role || null, email: email || null, phone: phone || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Role"><Input value={role} onChange={(e) => setRole(e.target.value)} placeholder="decision_maker, technical…" /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Email"><Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} /></Field>
          <Field label="Phone"><Input value={phone} onChange={(e) => setPhone(e.target.value)} /></Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================== Activities ===

type Activity = { id: string; activity_type: string; subject: string; related_type: string; related_id: string; done: boolean; notes?: string | null };

export function ActivitiesPage() {
  const { data, err, reload } = useList<Activity>("/crm/activities", "activities");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Activity | null>(null);

  async function create(body: Partial<Activity>) {
    await call(() => api.post<Activity>("/crm/activities", { id: "", activity_type: "TASK", subject: "", related_type: "", related_id: "", done: false, ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Activity>) {
    await call(() => api.patch<Activity>(`/crm/activities/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this activity?")) return;
    await call(() => api.del<void>(`/crm/activities/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Activities" description="Calls / Emails / Meetings / Tasks on any CRM record."
        actions={<Button onClick={() => setOpen(true)}>New activity</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No activities." /> : (
        <Table head={["Subject", "Type", "Related", "Done", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.subject}</Td>
              <Td><Badge>{r.activity_type}</Badge></Td>
              <Td className="font-mono text-xs">{r.related_type}:{r.related_id.slice(0, 8)}</Td>
              <Td><Badge tone={r.done ? "green" : "zinc"}>{String(r.done)}</Badge></Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <ActivityForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New activity" />
      <ActivityForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit activity`} />
    </div>
  );
}

function ActivityForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Activity; onClose: () => void;
  onSubmit: (b: Partial<Activity>) => Promise<void> | void; title: string;
}) {
  const [type, setType] = useState("TASK"); const [subject, setSubject] = useState("");
  const [relType, setRelType] = useState(""); const [relId, setRelId] = useState("");
  const [done, setDone] = useState(false); const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setType(initial?.activity_type ?? "TASK"); setSubject(initial?.subject ?? "");
      setRelType(initial?.related_type ?? ""); setRelId(initial?.related_id ?? "");
      setDone(initial?.done ?? false); setNotes(initial?.notes ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ activity_type: type, subject, related_type: relType, related_id: relId, done, notes: notes || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={440}>
      <form onSubmit={submit} className="space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <Field label="Type">
            <select value={type} onChange={(e) => setType(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["TASK", "CALL", "EMAIL", "MEETING"].map((s) => <option key={s} value={s}>{s}</option>)}
            </select>
          </Field>
          <label className="flex items-center gap-2 pt-6 text-sm">
            <input type="checkbox" checked={done} onChange={(e) => setDone(e.target.checked)} /> Done
          </label>
        </div>
        <Field label="Subject"><Input required value={subject} onChange={(e) => setSubject(e.target.value)} /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Related Type"><Input value={relType} onChange={(e) => setRelType(e.target.value)} placeholder="lead, customer…" /></Field>
          <Field label="Related ID"><Input value={relId} onChange={(e) => setRelId(e.target.value)} placeholder="record id" /></Field>
        </div>
        <Field label="Notes"><Textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ================================================================== Quotes ===

type Quote = { id: string; quote_number: string; status: string; total?: number | null; sales_order_id?: string | null };

export function QuotesPage() {
  const { data, err, reload } = useList<Quote>("/crm/quotes", "quotes");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Quote | null>(null);

  async function create(body: Partial<Quote>) {
    await call(() => api.post<Quote>("/crm/quotes", { id: "", quote_number: "", status: "DRAFT", total: 0, ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Quote>) {
    await call(() => api.patch<Quote>(`/crm/quotes/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this quote?")) return;
    await call(() => api.del<void>(`/crm/quotes/${id}`));
    await reload();
  }
  async function accept(id: string) {
    try {
      const r = await api.post<{ sales_order: Record<string, string> }>(`/crm/quotes/${id}/accept`, {});
      alert(`Quote accepted → ${r.sales_order.so_number}`);
      await reload();
    } catch (err) { alert(String(err)); }
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Quotes" description="Quote → Accept → Sales Order (ERP link)."
        actions={<Button onClick={() => setOpen(true)}>New quote</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No quotes." /> : (
        <Table head={["Quote #", "Status", "Total", "Sales Order", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{r.quote_number}</Td>
              <Td><Badge tone={r.status === "ACCEPTED" ? "green" : "zinc"}>{r.status}</Badge></Td>
              <Td>{r.total != null ? `$${r.total.toFixed(2)}` : "—"}</Td>
              <Td className="font-mono text-xs">{r.sales_order_id?.slice(0, 8) ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                {!r.sales_order_id && <Button className="py-1 text-xs" onClick={() => accept(r.id)}>Accept → SO</Button>}
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <QuoteForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New quote" />
      <QuoteForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.quote_number ?? ""}`} />
    </div>
  );
}

function QuoteForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Quote; onClose: () => void;
  onSubmit: (b: Partial<Quote>) => Promise<void> | void; title: string;
}) {
  const [total, setTotal] = useState(""); const [status, setStatus] = useState("DRAFT");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setTotal(initial?.total?.toString() ?? ""); setStatus(initial?.status ?? "DRAFT"); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ total: total ? parseFloat(total) : 0, status }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Total"><Input type="number" min={0} step="any" value={total} onChange={(e) => setTotal(e.target.value)} /></Field>
        <Field label="Status">
          <select value={status} onChange={(e) => setStatus(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {["DRAFT", "SENT", "ACCEPTED", "REJECTED", "EXPIRED"].map((s) => <option key={s} value={s}>{s}</option>)}
          </select>
        </Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

export default CrmLayout;
