import { useEffect, useState, useMemo } from "react";
import { NavLink, Outlet, useLocation, useNavigate, useParams } from "react-router";
import { api } from "@gateway/lib";
import {
  Alert, Badge, Button, Card, EmptyState, Field, Input, PageHeader,
  SidePanel, Spinner, Table, Td, Textarea, cn,
} from "@gateway/ui";
import { StoragePicker } from "@gateway/ui/storage-picker";
import { PassPreview, PKPassData, getPassStructure } from "./PassPreview";

type SubNav = { to: string; label: string; desc: string };
const DMC_NAV: SubNav[] = [
  { to: "/dmc/destinations", label: "Destinations", desc: "Cities & regions" },
  { to: "/dmc/packages", label: "Packages", desc: "Tour products" },
  { to: "/dmc/activities", label: "Activities", desc: "Things to do" },
  { to: "/dmc/accommodation", label: "Accommodation", desc: "Hotels & lodges" },
  { to: "/dmc/transport", label: "Transport", desc: "Vehicles & transfers" },
  { to: "/dmc/guides", label: "Guides", desc: "Tour guides" },
  { to: "/dmc/trips", label: "Trips", desc: "Booked itineraries" },
  { to: "/dmc/quotes", label: "Quotes", desc: "Price quotes" },
  { to: "/dmc/bookings", label: "Bookings", desc: "Confirmed bookings" },
  { to: "/dmc/catalog", label: "Catalog", desc: "Unified products" },
  { to: "/dmc/incidents", label: "Incidents", desc: "Issues & alerts" },
  { to: "/dmc/supplier-categories", label: "Supplier Categories", desc: "Classify vendors" },
  { to: "/dmc/wallet", label: "Wallet", desc: "PassKit designer & issuer" },
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

function todayIso() { return new Date().toISOString().slice(0, 10); }

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try { return await fn(); }
  catch (e) { alert(e instanceof Error ? e.message : String(e)); throw e; }
}

// ---------------------------------------------------------------- layout ---

export function DmcLayout() {
  const loc = useLocation();
  return (
    <div className="flex gap-6">
      <aside className="hidden w-[210px] shrink-0 lg:block"
        style={{ position: "sticky", top: 80, alignSelf: "flex-start", height: "calc(100dvh - 96px)" }}>
        <div className="rounded-xl border border-[var(--app-border)] bg-[var(--app-card)] p-3">
          <div className="px-2 pb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">DMC</div>
          <nav className="flex flex-col gap-0.5">
            {DMC_NAV.map((i) => (
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
          {DMC_NAV.map((i) => (
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
        {loc.pathname === "/dmc" && <DmcOverview />}
      </div>
    </div>
  );
}

export function DmcOverview() {
  const nav = useNavigate();
  return (
    <div className="space-y-6">
      <PageHeader title="DMC" description="Destinations → Packages → Itineraries → Trips → Quotes → Bookings" />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {DMC_NAV.map((i) => (
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

// ============================================================ Destinations ===

type Destination = { id: string; name: string; country: string; region?: string | null; city?: string | null; lat?: number | null; lon?: number | null; timezone?: string | null; description?: string | null };

export function DestinationsPage() {
  const { data, err, reload } = useList<Destination>("/dmc/destinations", "destinations");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Destination | null>(null);

  async function create(body: Partial<Destination>) {
    await call(() => api.post<Destination>("/dmc/destinations", { name: "", country: "", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Destination>) {
    await call(() => api.patch<Destination>(`/dmc/destinations/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this destination?")) return;
    await call(() => api.del<void>(`/dmc/destinations/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Destinations" description="Cities, regions and countries used across itineraries."
        actions={<Button onClick={() => setOpen(true)}>New destination</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No destinations yet." /> : (
        <Table head={["Name", "Country", "Region", "Coords", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td><Badge>{r.country}</Badge></Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">{r.region ?? "—"}</Td>
              <Td className="font-mono text-xs">{r.lat && r.lon ? `${r.lat.toFixed(3)}, ${r.lon.toFixed(3)}` : "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <DestinationForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New destination" />
      <DestinationForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function DestinationForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Destination; onClose: () => void;
  onSubmit: (b: Partial<Destination>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [country, setCountry] = useState("");
  const [region, setRegion] = useState(""); const [city, setCity] = useState("");
  const [lat, setLat] = useState(""); const [lon, setLon] = useState("");
  const [timezone, setTimezone] = useState(""); const [description, setDescription] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setCountry(initial?.country ?? ""); setRegion(initial?.region ?? "");
      setCity(initial?.city ?? ""); setLat(initial?.lat?.toString() ?? ""); setLon(initial?.lon?.toString() ?? "");
      setTimezone(initial?.timezone ?? ""); setDescription(initial?.description ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, country, region: region || null, city: city || null,
      lat: lat ? parseFloat(lat) : null, lon: lon ? parseFloat(lon) : null,
      timezone: timezone || null, description: description || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={460}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Country"><Input required value={country} onChange={(e) => setCountry(e.target.value)} placeholder="TZ, KE, UG…" /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Region"><Input value={region} onChange={(e) => setRegion(e.target.value)} /></Field>
          <Field label="City"><Input value={city} onChange={(e) => setCity(e.target.value)} /></Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Latitude"><Input type="number" step="any" value={lat} onChange={(e) => setLat(e.target.value)} /></Field>
          <Field label="Longitude"><Input type="number" step="any" value={lon} onChange={(e) => setLon(e.target.value)} /></Field>
        </div>
        <Field label="Timezone"><Input value={timezone} onChange={(e) => setTimezone(e.target.value)} placeholder="Africa/Dar_es_Salaam" /></Field>
        <Field label="Description"><Textarea rows={3} value={description} onChange={(e) => setDescription(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ================================================================ Packages ===

type Package = { id: string; name: string; duration_days: number; base_price: number; currency: string; description?: string | null };

export function PackagesPage() {
  const { data, err, reload } = useList<Package>("/dmc/packages", "packages");
  const nav = useNavigate();
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Package | null>(null);

  async function create(body: Partial<Package>) {
    await call(() => api.post<Package>("/dmc/packages", { name: "", duration_days: 1, base_price: 0, currency: "TZS", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Package>) {
    await call(() => api.patch<Package>(`/dmc/packages/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this package?")) return;
    await call(() => api.del<void>(`/dmc/packages/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Packages" description="Tour products with duration and base pricing."
        actions={<Button onClick={() => setOpen(true)}>New package</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No packages yet." /> : (
        <Table head={["Name", "Days", "Price", "Currency", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td>{r.duration_days}</Td>
              <Td>{r.base_price.toFixed(2)}</Td>
              <Td><Badge>{r.currency}</Badge></Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => nav(`/dmc/packages/${r.id}/itineraries`)}>Itineraries</Button>
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <PackageForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New package" />
      <PackageForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function PackageForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Package; onClose: () => void;
  onSubmit: (b: Partial<Package>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [days, setDays] = useState(1);
  const [price, setPrice] = useState(0); const [currency, setCurrency] = useState("TZS");
  const [desc, setDesc] = useState(""); const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setDays(initial?.duration_days ?? 1);
      setPrice(initial?.base_price ?? 0); setCurrency(initial?.currency ?? "TZS"); setDesc(initial?.description ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, duration_days: days, base_price: price, currency, description: desc || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <div className="grid grid-cols-3 gap-3">
          <Field label="Days"><Input type="number" min={1} value={days} onChange={(e) => setDays(Number(e.target.value))} /></Field>
          <Field label="Price"><Input type="number" min={0} step="any" value={price} onChange={(e) => setPrice(Number(e.target.value))} /></Field>
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
        </div>
        <Field label="Description"><Textarea rows={3} value={desc} onChange={(e) => setDesc(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================== Activities ===

type Activity = { id: string; name: string; category?: string | null; duration_minutes: number; price: number; currency: string; description?: string | null };

export function ActivitiesPage() {
  const { data, err, reload } = useList<Activity>("/dmc/activities", "activities");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Activity | null>(null);

  async function create(body: Partial<Activity>) {
    await call(() => api.post<Activity>("/dmc/activities", { name: "", duration_minutes: 60, price: 0, currency: "TZS", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Activity>) {
    await call(() => api.patch<Activity>(`/dmc/activities/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this activity?")) return;
    await call(() => api.del<void>(`/dmc/activities/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Activities" description="Tour activities with pricing and duration."
        actions={<Button onClick={() => setOpen(true)}>New activity</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No activities yet." /> : (
        <Table head={["Name", "Category", "Duration", "Price", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td><Badge>{r.category ?? "—"}</Badge></Td>
              <Td>{r.duration_minutes} min</Td>
              <Td>{r.price.toFixed(2)} {r.currency}</Td>
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
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function ActivityForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Activity; onClose: () => void;
  onSubmit: (b: Partial<Activity>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [cat, setCat] = useState("");
  const [dur, setDur] = useState(60); const [price, setPrice] = useState(0);
  const [currency, setCurrency] = useState("TZS"); const [desc, setDesc] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setCat(initial?.category ?? ""); setDur(initial?.duration_minutes ?? 60);
      setPrice(initial?.price ?? 0); setCurrency(initial?.currency ?? "TZS"); setDesc(initial?.description ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, category: cat || null, duration_minutes: dur, price, currency, description: desc || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Category"><Input value={cat} onChange={(e) => setCat(e.target.value)} placeholder="safari, cultural, adventure…" /></Field>
        <div className="grid grid-cols-3 gap-3">
          <Field label="Duration (min)"><Input type="number" min={0} value={dur} onChange={(e) => setDur(Number(e.target.value))} /></Field>
          <Field label="Price"><Input type="number" min={0} step="any" value={price} onChange={(e) => setPrice(Number(e.target.value))} /></Field>
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
        </div>
        <Field label="Description"><Textarea rows={3} value={desc} onChange={(e) => setDesc(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// =========================================================== Accommodation ===

type Accommodation = { id: string; name: string; room_type?: string | null; capacity: number; nightly_rate: number; currency: string };

export function AccommodationPage() {
  const { data, err, reload } = useList<Accommodation>("/dmc/accommodation", "accommodation");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Accommodation | null>(null);

  async function create(body: Partial<Accommodation>) {
    await call(() => api.post<Accommodation>("/dmc/accommodation", { name: "", capacity: 2, nightly_rate: 0, currency: "TZS", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Accommodation>) {
    await call(() => api.patch<Accommodation>(`/dmc/accommodation/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this accommodation?")) return;
    await call(() => api.del<void>(`/dmc/accommodation/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Accommodation" description="Hotels, lodges and camps with nightly rates."
        actions={<Button onClick={() => setOpen(true)}>New accommodation</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No accommodation yet." /> : (
        <Table head={["Name", "Room Type", "Capacity", "Nightly Rate", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td><Badge>{r.room_type ?? "—"}</Badge></Td>
              <Td>{r.capacity}</Td>
              <Td>{r.nightly_rate.toFixed(2)} {r.currency}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <AccommodationForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New accommodation" />
      <AccommodationForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function AccommodationForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Accommodation; onClose: () => void;
  onSubmit: (b: Partial<Accommodation>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [roomType, setRoomType] = useState("");
  const [cap, setCap] = useState(2); const [rate, setRate] = useState(0);
  const [currency, setCurrency] = useState("TZS"); const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setRoomType(initial?.room_type ?? "");
      setCap(initial?.capacity ?? 2); setRate(initial?.nightly_rate ?? 0); setCurrency(initial?.currency ?? "TZS"); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, room_type: roomType || null, capacity: cap, nightly_rate: rate, currency }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Room Type"><Input value={roomType} onChange={(e) => setRoomType(e.target.value)} placeholder="single, double, suite…" /></Field>
        <div className="grid grid-cols-3 gap-3">
          <Field label="Capacity"><Input type="number" min={1} value={cap} onChange={(e) => setCap(Number(e.target.value))} /></Field>
          <Field label="Rate"><Input type="number" min={0} step="any" value={rate} onChange={(e) => setRate(Number(e.target.value))} /></Field>
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ================================================================ Transport ===

type Transport = { id: string; mode: string; capacity: number; hourly_rate: number; currency: string };

export function TransportPage() {
  const { data, err, reload } = useList<Transport>("/dmc/transport", "transport");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Transport | null>(null);

  async function create(body: Partial<Transport>) {
    await call(() => api.post<Transport>("/dmc/transport", { mode: "", capacity: 4, hourly_rate: 0, currency: "TZS", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Transport>) {
    await call(() => api.patch<Transport>(`/dmc/transport/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this transport?")) return;
    await call(() => api.del<void>(`/dmc/transport/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Transport" description="Vehicles and transfer services."
        actions={<Button onClick={() => setOpen(true)}>New transport</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No transport yet." /> : (
        <Table head={["Mode", "Capacity", "Hourly Rate", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.mode}</Td>
              <Td>{r.capacity}</Td>
              <Td>{r.hourly_rate.toFixed(2)} {r.currency}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <TransportForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New transport" />
      <TransportForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.mode ?? ""}`} />
    </div>
  );
}

function TransportForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Transport; onClose: () => void;
  onSubmit: (b: Partial<Transport>) => Promise<void> | void; title: string;
}) {
  const [mode, setMode] = useState(""); const [cap, setCap] = useState(4);
  const [rate, setRate] = useState(0); const [currency, setCurrency] = useState("TZS");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setMode(initial?.mode ?? ""); setCap(initial?.capacity ?? 4);
      setRate(initial?.hourly_rate ?? 0); setCurrency(initial?.currency ?? "TZS"); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ mode, capacity: cap, hourly_rate: rate, currency }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={380}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Mode"><Input required value={mode} onChange={(e) => setMode(e.target.value)} placeholder="van, bus, 4x4, boat…" /></Field>
        <div className="grid grid-cols-3 gap-3">
          <Field label="Capacity"><Input type="number" min={1} value={cap} onChange={(e) => setCap(Number(e.target.value))} /></Field>
          <Field label="Rate/hr"><Input type="number" min={0} step="any" value={rate} onChange={(e) => setRate(Number(e.target.value))} /></Field>
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ================================================================= Guides ===

type Guide = { id: string; full_name: string; languages?: string | null; daily_rate: number; currency: string; phone?: string | null; email?: string | null };

export function GuidesPage() {
  const { data, err, reload } = useList<Guide>("/dmc/guides", "guides");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Guide | null>(null);

  async function create(body: Partial<Guide>) {
    await call(() => api.post<Guide>("/dmc/guides", { full_name: "", daily_rate: 0, currency: "TZS", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Guide>) {
    await call(() => api.patch<Guide>(`/dmc/guides/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this guide?")) return;
    await call(() => api.del<void>(`/dmc/guides/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Guides" description="Tour guides with language skills and daily rates."
        actions={<Button onClick={() => setOpen(true)}>New guide</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No guides yet." /> : (
        <Table head={["Name", "Languages", "Daily Rate", "Phone", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.full_name}</Td>
              <Td className="text-xs">{r.languages ?? "—"}</Td>
              <Td>{r.daily_rate.toFixed(2)} {r.currency}</Td>
              <Td className="text-xs">{r.phone ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <GuideForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New guide" />
      <GuideForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.full_name ?? ""}`} />
    </div>
  );
}

function GuideForm({ open, initial, onClose, onSubmit, title }: {
  open: boolean; initial?: Guide; onClose: () => void;
  onSubmit: (b: Partial<Guide>) => Promise<void> | void; title: string;
}) {
  const [name, setName] = useState(""); const [langs, setLangs] = useState("");
  const [rate, setRate] = useState(0); const [currency, setCurrency] = useState("TZS");
  const [phone, setPhone] = useState(""); const [email, setEmail] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.full_name ?? ""); setLangs(initial?.languages ?? "");
      setRate(initial?.daily_rate ?? 0); setCurrency(initial?.currency ?? "TZS");
      setPhone(initial?.phone ?? ""); setEmail(initial?.email ?? ""); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ full_name: name, languages: langs || null, daily_rate: rate, currency, phone: phone || null, email: email || null }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Full Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Languages"><Input value={langs} onChange={(e) => setLangs(e.target.value)} placeholder="English, Swahili…" /></Field>
        <div className="grid grid-cols-3 gap-3">
          <Field label="Daily Rate"><Input type="number" min={0} step="any" value={rate} onChange={(e) => setRate(Number(e.target.value))} /></Field>
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
          <div />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Phone"><Input value={phone} onChange={(e) => setPhone(e.target.value)} /></Field>
          <Field label="Email"><Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} /></Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// =================================================================== Trips ===

type Trip = { id: string; name: string; package_id?: string | null; start_date: string; end_date: string; pax_count: number; status: string };

export function TripsPage() {
  const { data, err, reload } = useList<Trip>("/dmc/trips", "trips");
  const packages = useList<Package>("/dmc/packages", "packages");
  const nav = useNavigate();
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Trip | null>(null);

  async function create(body: Partial<Trip>) {
    await call(() => api.post<Trip>("/dmc/trips", { name: "", start_date: todayIso(), end_date: todayIso(), pax_count: 2, status: "draft", ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Trip>) {
    await call(() => api.patch<Trip>(`/dmc/trips/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this trip?")) return;
    await call(() => api.del<void>(`/dmc/trips/${id}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Trips" description="Booked itineraries with services, quotes and bookings."
        actions={<Button onClick={() => setOpen(true)}>New trip</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No trips yet." /> : (
        <Table head={["Name", "Dates", "Pax", "Status", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td className="text-xs">{r.start_date} → {r.end_date}</Td>
              <Td>{r.pax_count}</Td>
              <Td><Badge tone={r.status === "confirmed" ? "green" : r.status === "cancelled" ? "red" : undefined}>{r.status}</Badge></Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => nav(`/dmc/trips/${r.id}`)}>View</Button>
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(r)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <TripForm open={open} onClose={() => setOpen(false)} onSubmit={create} packages={packages.data ?? []} title="New trip" />
      <TripForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)}
        onSubmit={async (b) => { if (edit) await update(edit.id, b); }} packages={packages.data ?? []} title={`Edit ${edit?.name ?? ""}`} />
    </div>
  );
}

function TripForm({ open, initial, onClose, onSubmit, packages, title }: {
  open: boolean; initial?: Trip; onClose: () => void;
  onSubmit: (b: Partial<Trip>) => Promise<void> | void;
  packages: { id: string; name: string }[]; title: string;
}) {
  const [name, setName] = useState(""); const [pkgId, setPkgId] = useState("");
  const [start, setStart] = useState(todayIso()); const [end, setEnd] = useState(todayIso());
  const [pax, setPax] = useState(2); const [status, setStatus] = useState("draft");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (open) { setName(initial?.name ?? ""); setPkgId(initial?.package_id ?? "");
      setStart(initial?.start_date ?? todayIso()); setEnd(initial?.end_date ?? todayIso());
      setPax(initial?.pax_count ?? 2); setStatus(initial?.status ?? "draft"); }
  }, [open, initial]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ name, package_id: pkgId || null, start_date: start, end_date: end, pax_count: pax, status }); }
    finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={460}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Name"><Input required value={name} onChange={(e) => setName(e.target.value)} /></Field>
        <Field label="Package">
          <select value={pkgId} onChange={(e) => setPkgId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            <option value="">— none —</option>
            {packages.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Start date"><Input type="date" value={start} onChange={(e) => setStart(e.target.value)} /></Field>
          <Field label="End date"><Input type="date" value={end} onChange={(e) => setEnd(e.target.value)} /></Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Pax"><Input type="number" min={1} value={pax} onChange={(e) => setPax(Number(e.target.value))} /></Field>
          <Field label="Status">
            <select value={status} onChange={(e) => setStatus(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["draft", "quoted", "confirmed", "completed", "cancelled"].map((s) => <option key={s} value={s}>{s}</option>)}
            </select>
          </Field>
        </div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ========================================================= Supplier Categories ===

type SupplierCategory = { supplier_id: string; category: string; destination_id?: string | null };

export function SupplierCategoriesPage() {
  const { data, err, reload } = useList<SupplierCategory>("/dmc/supplier-categories", "supplier_categories");
  const [open, setOpen] = useState(false);

  async function upsert(supplierId: string, category: string) {
    await call(() => api.post<SupplierCategory>("/dmc/supplier-categories", { supplier_id: supplierId, category, destination_id: null }));
    setOpen(false); await reload();
  }
  async function del(supplierId: string, category: string) {
    if (!confirm("Remove this category?")) return;
    await call(() => api.del<void>(`/dmc/supplier-categories/${supplierId}/${category}`));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Supplier Categories" description="Classify ERP suppliers by DMC service type."
        actions={<Button onClick={() => setOpen(true)}>Add category</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No supplier categories." /> : (
        <Table head={["Supplier ID", "Category", ""]}>
          {data.map((r) => (
            <tr key={`${r.supplier_id}-${r.category}`} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{r.supplier_id.slice(0, 8)}</Td>
              <Td><Badge>{r.category}</Badge></Td>
              <Td className="text-right">
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.supplier_id, r.category)}>Remove</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <SupplierCategoryForm open={open} onClose={() => setOpen(false)} onSubmit={upsert} />
    </div>
  );
}

function SupplierCategoryForm({ open, onClose, onSubmit }: {
  open: boolean; onClose: () => void; onSubmit: (sid: string, cat: string) => Promise<void> | void;
}) {
  const [sid, setSid] = useState(""); const [cat, setCat] = useState(""); const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setSid(""); setCat(""); } }, [open]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); if (!sid || !cat) return; setBusy(true);
    try { await onSubmit(sid, cat); } finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="Add supplier category" width={380}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Supplier ID"><Input required value={sid} onChange={(e) => setSid(e.target.value)} placeholder="ERP supplier id" /></Field>
        <Field label="Category"><Input required value={cat} onChange={(e) => setCat(e.target.value)} placeholder="transport, accommodation, guide…" /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Add</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================ Itineraries ===

type Itinerary = { id: string; package_id: string; day_number: number; title: string; notes?: string | null };
type ItineraryStop = { id: string; itinerary_id: string; day_number: number; hour: string; location_id?: string | null; activity?: string | null; notes?: string | null };

export function ItinerariesPage() {
  const { pkgId } = useParams<{ pkgId: string }>();
  const { data: packages } = useList<{ id: string; name: string }>("/dmc/packages", "packages");
  const [selectedPkg, setSelectedPkg] = useState(pkgId ?? "");
  const { data, err, reload } = useList<Itinerary>(selectedPkg ? `/dmc/packages/${selectedPkg}/itineraries` : "", "itineraries");
  const [open, setOpen] = useState(false);
  const [stopsFor, setStopsFor] = useState<string | null>(null);
  const [addStopFor, setAddStopFor] = useState<string | null>(null);
  const { data: stops } = useList<ItineraryStop>(stopsFor ? `/dmc/itineraries/${stopsFor}/stops` : "", "stops");

  async function create(body: Partial<Itinerary>) {
    await call(() => api.post("/dmc/itineraries", { package_id: selectedPkg, day_number: 1, title: "", ...body }));
    setOpen(false); await reload();
  }
  async function del(id: string) {
    if (!confirm("Delete this itinerary?")) return;
    await call(() => api.del(`/dmc/itineraries/${id}`)); await reload();
  }
  async function delStop(id: string) {
    if (!confirm("Delete this stop?")) return;
    await call(() => api.del(`/dmc/itinerary-stops/${id}`));
    if (stopsFor) { await reload(); }
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Itineraries" description="Day-by-day plans for packages."
        actions={<Button onClick={() => setOpen(true)} disabled={!selectedPkg}>New itinerary</Button>} />
      <Field label="Package">
        <select value={selectedPkg} onChange={(e) => setSelectedPkg(e.target.value)}
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
          <option value="">Select a package…</option>
          {packages?.map((p) => <option key={p.id} value={p.id}>{p.name}</option>)}
        </select>
      </Field>
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No itineraries for this package." /> : (
        <Table head={["Day", "Title", "Notes", ""]}>
          {data.map((r) => (
            <tr key={r.id} className="hover:bg-[var(--sb-hover)]">
              <Td><Badge>Day {r.day_number}</Badge></Td>
              <Td className="font-medium">{r.title}</Td>
              <Td className="text-xs text-[var(--app-fg-muted)] max-w-[200px] truncate">{r.notes ?? "—"}</Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setStopsFor(stopsFor === r.id ? null : r.id)}>
                  {stopsFor === r.id ? "Hide stops" : "Stops"}
                </Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(r.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      {stopsFor && stops && (
        <Card className="p-4 space-y-2">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-semibold">Stops for itinerary</h4>
            <Button variant="ghost" className="py-1 text-xs" onClick={() => { setStopsFor(null); setAddStopFor(null); }}>Hide</Button>
          </div>
          {stops.length === 0 ? <p className="text-xs text-[var(--app-fg-muted)]">No stops yet.</p> : (
            <Table head={["Hour", "Activity", "Location", "Notes", ""]}>
              {stops.map((s) => (
                <tr key={s.id} className="hover:bg-[var(--sb-hover)]">
                  <Td className="font-mono text-xs">{s.hour}</Td>
                  <Td>{s.activity ?? "—"}</Td>
                  <Td className="text-xs">{s.location_id ?? "—"}</Td>
                  <Td className="text-xs text-[var(--app-fg-muted)]">{s.notes ?? "—"}</Td>
                  <Td className="text-right">
                    <Button variant="ghost" className="py-1 text-xs" onClick={() => delStop(s.id)}>Delete</Button>
                  </Td>
                </tr>
              ))}
            </Table>
          )}
          <Button variant="ghost" className="text-xs" onClick={() => setAddStopFor(addStopFor === stopsFor ? null : stopsFor)}>
            + Add stop
          </Button>
          {addStopFor && <StopForm itineraryId={addStopFor} onCreated={async () => { await reload(); }} />}
        </Card>
      )}
      <ItineraryForm open={open} onClose={() => setOpen(false)} onSubmit={create} />
    </div>
  );
}

function ItineraryForm({ open, onClose, onSubmit }: {
  open: boolean; onClose: () => void; onSubmit: (b: Partial<Itinerary>) => Promise<void> | void;
}) {
  const [day, setDay] = useState(1); const [title, setTitle] = useState(""); const [notes, setNotes] = useState(""); const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setDay(1); setTitle(""); setNotes(""); } }, [open]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ day_number: day, title, notes: notes || null }); } finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="New itinerary" width={400}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Day number"><Input type="number" min={1} value={day} onChange={(e) => setDay(Number(e.target.value))} /></Field>
        <Field label="Title"><Input required value={title} onChange={(e) => setTitle(e.target.value)} placeholder="e.g. Arrival in Arusha" /></Field>
        <Field label="Notes"><Textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Save</Button>
        </div>
      </form>
    </SidePanel>
  );
}

function StopForm({ itineraryId, onCreated }: { itineraryId: string; onCreated: () => Promise<void> | void }) {
  const [hour, setHour] = useState("08:00");
  const [activity, setActivity] = useState("");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try {
      const hourNum = parseInt(hour.split(":")[0] || "8", 10) * 60 + parseInt(hour.split(":")[1] || "0", 10);
      await call(() => api.post(`/dmc/itineraries/${itineraryId}/stops`, { day_number: 1, hour: hourNum, activity: activity || null, notes: notes || null }));
      setActivity(""); setNotes(""); await onCreated();
    } finally { setBusy(false); }
  }
  return (
    <form onSubmit={submit} className="mt-2 space-y-3 rounded-lg border border-[var(--app-border)] bg-[var(--app-bg)] p-3">
      <div className="grid grid-cols-2 gap-3">
        <Field label="Time (HH:MM)"><Input value={hour} onChange={(e) => setHour(e.target.value)} placeholder="08:00" /></Field>
        <Field label="Activity"><Input value={activity} onChange={(e) => setActivity(e.target.value)} placeholder="e.g. Game drive" /></Field>
      </div>
      <Field label="Notes"><Input value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
      <Button type="submit" className="py-1 text-xs" loading={busy}>Add stop</Button>
    </form>
  );
}

// ============================================================ Quotes ===

type Quote = { id: string; trip_id: string; version: number; currency: string; subtotal: number; tax: number; total: number; status: string; sent_at?: string | null; accepted_at?: string | null; created_at: string };

export function QuotesPage() {
  const { data: trips, err: tripsErr } = useList<Trip>("/dmc/trips", "trips");
  const [quotes, setQuotes] = useState<Quote[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [open, setOpen] = useState(false);

  async function loadAllQuotes() {
    setBusy(true);
    try {
      const tripsRes = await api.get<{ trips: Trip[] }>("/dmc/trips");
      const all: Quote[] = [];
      for (const t of tripsRes.trips) {
        try {
          const r = await api.get<{ quotes: Quote[] }>(`/dmc/trips/${t.id}/quotes`);
          all.push(...r.quotes.map((q) => ({ ...q, _tripName: t.name } as Quote & { _tripName: string })));
        } catch { /* skip */ }
      }
      setQuotes(all);
    } catch { /* ignore */ }
    setBusy(false);
  }
  useEffect(() => { void loadAllQuotes(); }, []);

  async function accept(id: string) {
    await call(() => api.post(`/dmc/quotes/${id}/accept`)); await loadAllQuotes();
  }
  async function create(body: { trip_id: string; version: number; currency: string; subtotal: number; tax: number; total: number }) {
    await call(() => api.post(`/dmc/trips/${body.trip_id}/quotes`, { version: 1, currency: "TZS", subtotal: 0, tax: 0, total: 0, ...body }));
    setOpen(false); await loadAllQuotes();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Quotes" description="Price quotes for trips." actions={
        <div className="flex gap-2">
          <Button onClick={() => setOpen(true)}>New quote</Button>
          <Button onClick={() => void loadAllQuotes()} loading={busy}>Refresh</Button>
        </div>} />
      {tripsErr && <Alert>{tripsErr}</Alert>}
      {!quotes ? <Spinner /> : quotes.length === 0 ? <EmptyState message="No quotes yet." /> : (
        <Table head={["Trip", "Version", "Total", "Status", "Accepted", ""]}>
          {quotes.map((q) => (
            <tr key={q.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium text-xs">{(q as Quote & { _tripName?: string })._tripName ?? q.trip_id.slice(0, 8)}</Td>
              <Td><Badge>v{q.version}</Badge></Td>
              <Td className="font-medium">{q.currency} {q.total.toFixed(2)}</Td>
              <Td><Badge tone={q.status === "accepted" ? "green" : q.status === "sent" ? "blue" : "zinc"}>{q.status}</Badge></Td>
              <Td className="text-xs text-[var(--app-fg-muted)]">{q.accepted_at ? new Date(q.accepted_at).toLocaleDateString() : "—"}</Td>
              <Td className="text-right">
                {q.status !== "accepted" && (
                  <Button variant="ghost" className="py-1 text-xs" onClick={() => void accept(q.id)}>Accept</Button>
                )}
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <QuoteForm open={open} onClose={() => setOpen(false)} trips={trips ?? []} onSubmit={create} />
    </div>
  );
}

function QuoteForm({ open, onClose, trips, onSubmit }: {
  open: boolean; onClose: () => void; trips: Trip[];
  onSubmit: (b: { trip_id: string; version: number; currency: string; subtotal: number; tax: number; total: number }) => Promise<void> | void;
}) {
  const [tripId, setTripId] = useState(""); const [currency, setCurrency] = useState("TZS");
  const [subtotal, setSubtotal] = useState(0); const [tax, setTax] = useState(0);
  const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setTripId(trips[0]?.id ?? ""); setCurrency("TZS"); setSubtotal(0); setTax(0); } }, [open, trips]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ trip_id: tripId, version: 1, currency, subtotal, tax, total: subtotal + tax }); } finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="New quote" width={440}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Trip">
          <select value={tripId} onChange={(e) => setTripId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {trips.map((t) => <option key={t.id} value={t.id}>{t.name}</option>)}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
          <div />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Subtotal"><Input type="number" min={0} step={0.01} value={subtotal} onChange={(e) => setSubtotal(Number(e.target.value))} /></Field>
          <Field label="Tax"><Input type="number" min={0} step={0.01} value={tax} onChange={(e) => setTax(Number(e.target.value))} /></Field>
        </div>
        <div className="text-sm text-[var(--app-fg-muted)]">Total: {currency} {(subtotal + tax).toFixed(2)}</div>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Create quote</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================ Bookings ===

type Booking = { id: string; quote_id: string; status: string; deposit_paid: number; balance_due: number; currency: string; total: number; confirmed_at?: string | null; created_at: string };

export function BookingsPage() {
  const [bookings, setBookings] = useState<Booking[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [open, setOpen] = useState(false);

  async function load() {
    setBusy(true);
    try {
      const tripsRes = await api.get<{ trips: Trip[] }>("/dmc/trips");
      const all: Booking[] = [];
      for (const t of tripsRes.trips) {
        try {
          const quotesRes = await api.get<{ quotes: Quote[] }>(`/dmc/trips/${t.id}/quotes`);
          for (const q of quotesRes.quotes) {
            try {
              const br = await api.get<{ bookings: Booking[] }>(`/dmc/quotes/${q.id}/bookings`);
              all.push(...br.bookings);
            } catch { /* skip */ }
          }
        } catch { /* skip */ }
      }
      setBookings(all);
    } catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
    setBusy(false);
  }
  useEffect(() => { void load(); }, []);

  async function confirm(id: string) {
    await call(() => api.post(`/dmc/bookings/${id}/confirm`)); await load();
  }
  async function create(body: { quote_id: string; total: number; deposit_paid: number; balance_due: number; currency: string }) {
    await call(() => api.post(`/dmc/quotes/${body.quote_id}/bookings`, body));
    setOpen(false); await load();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Bookings" description="Confirmed and pending bookings."
        actions={<div className="flex gap-2"><Button onClick={() => setOpen(true)}>New booking</Button><Button onClick={() => void load()} loading={busy}>Refresh</Button></div>} />
      {err && <Alert>{err}</Alert>}
      {!bookings ? <Spinner /> : bookings.length === 0 ? <EmptyState message="No bookings yet." /> : (
        <Table head={["Quote ID", "Total", "Deposit", "Balance", "Status", ""]}>
          {bookings.map((b) => (
            <tr key={b.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-mono text-xs">{b.quote_id.slice(0, 8)}</Td>
              <Td className="font-medium">{b.currency} {b.total.toFixed(2)}</Td>
              <Td className="text-xs">{b.deposit_paid.toFixed(2)}</Td>
              <Td className="text-xs">{b.balance_due.toFixed(2)}</Td>
              <Td><Badge tone={b.status === "confirmed" ? "green" : "zinc"}>{b.status}</Badge></Td>
              <Td className="text-right">
                {b.status !== "confirmed" && (
                  <Button variant="ghost" className="py-1 text-xs" onClick={() => void confirm(b.id)}>Confirm</Button>
                )}
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <BookingForm open={open} onClose={() => setOpen(false)} onSubmit={create} />
    </div>
  );
}

type BookingQuote = Quote & { _tripName?: string };

function BookingForm({ open, onClose, onSubmit }: {
  open: boolean; onClose: () => void;
  onSubmit: (b: { quote_id: string; total: number; deposit_paid: number; balance_due: number; currency: string }) => Promise<void> | void;
}) {
  const [quotes, setQuotes] = useState<BookingQuote[]>([]);
  const [quoteId, setQuoteId] = useState("");
  const [deposit, setDeposit] = useState(0);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    if (!open) return;
    api.get<{ trips: Trip[] }>("/dmc/trips").then((tr) => {
      const all: BookingQuote[] = [];
      Promise.all(tr.trips.map(async (t) => {
        try {
          const r = await api.get<{ quotes: Quote[] }>(`/dmc/trips/${t.id}/quotes`);
          r.quotes.forEach((q) => all.push({ ...q, _tripName: t.name } as BookingQuote));
        } catch { /* skip */ }
      })).then(() => { setQuotes(all); if (all.length) setQuoteId(all[0].id); });
    });
  }, [open]);
  const selected = quotes.find((q) => q.id === quoteId);
  useEffect(() => {
    if (selected) { setDeposit(selected.total * 0.3); }
  }, [selected]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try {
      const q = selected!;
      const total = q.total; const currency = q.currency;
      await onSubmit({ quote_id: quoteId, total, deposit_paid: deposit, balance_due: total - deposit, currency });
    } finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="New booking" width={440}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Quote">
          <select value={quoteId} onChange={(e) => setQuoteId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {quotes.map((q) => <option key={q.id} value={q.id}>{q._tripName ?? q.trip_id.slice(0, 8)} — {q.currency} {q.total.toFixed(2)} (v{q.version})</option>)}
          </select>
        </Field>
        {selected && (
          <>
            <div className="text-sm">Total: {selected.currency} {selected.total.toFixed(2)}</div>
            <Field label="Deposit paid">
              <Input type="number" min={0} step={0.01} value={deposit} onChange={(e) => setDeposit(Number(e.target.value))} />
            </Field>
            <div className="text-sm">Balance due: {selected.currency} {(selected.total - deposit).toFixed(2)}</div>
          </>
        )}
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy} disabled={!quoteId}>Create booking</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================ Catalog ===

type CatalogItem = { id: string; kind: string; name: string; description?: string | null; price?: number | null; currency?: string | null };

export function CatalogPage() {
  const { data, err } = useList<CatalogItem>("/dmc/catalog", "items");
  return (
    <div className="space-y-6">
      <PageHeader title="Catalog" description="Unified product catalog across ERP, activities and accommodation." />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No catalog items." /> : (
        <Table head={["Name", "Type", "Price", "Description"]}>
          {data.map((r) => (
            <tr key={`${r.kind}-${r.id}`} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{r.name}</Td>
              <Td><Badge>{r.kind}</Badge></Td>
              <Td className="text-xs">{r.price != null ? `${r.currency ?? ""} ${r.price.toFixed(2)}` : "—"}</Td>
              <Td className="text-xs text-[var(--app-fg-muted)] max-w-[300px] truncate">{r.description ?? "—"}</Td>
            </tr>
          ))}
        </Table>
      )}
    </div>
  );
}

// ============================================================ Incidents ===

type Incident = { id: string; trip_id: string; type_: string; severity: string; title: string; description?: string | null; assigned_to?: string | null; status: string; extra_cost: number; created_at: string; resolved_at?: string | null };

export function IncidentsPage() {
  const { data: trips } = useList<Trip>("/dmc/trips", "trips");
  const [incidents, setIncidents] = useState<Incident[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [open, setOpen] = useState(false);
  const [tripId, setTripId] = useState("");

  async function load() {
    setBusy(true);
    try {
      const tripsRes = await api.get<{ trips: Trip[] }>("/dmc/trips");
      const all: Incident[] = [];
      for (const t of tripsRes.trips) {
        try {
          const r = await api.get<{ incidents: Incident[] }>(`/dmc/trips/${t.id}/incidents`);
          all.push(...r.incidents);
        } catch { /* skip */ }
      }
      setIncidents(all);
    } catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
    setBusy(false);
  }
  useEffect(() => { void load(); }, []);

  async function resolve(id: string) {
    await call(() => api.post(`/dmc/incidents/${id}/resolve`)); await load();
  }
  async function createIncident(body: { title: string; type_: string; severity?: string | null; description?: string | null; assigned_to?: string | null; extra_cost?: number | null }) {
    await call(() => api.post(`/dmc/trips/${tripId}/incidents`, body));
    setOpen(false); await load();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Incidents" description="Issues, alerts and resolutions across trips."
        actions={<div className="flex gap-2"><Button onClick={() => setOpen(true)}>New incident</Button><Button onClick={() => void load()} loading={busy}>Refresh</Button></div>} />
      {err && <Alert>{err}</Alert>}
      {!incidents ? <Spinner /> : incidents.length === 0 ? <EmptyState message="No incidents." /> : (
        <Table head={["Title", "Type", "Severity", "Status", "Cost", ""]}>
          {incidents.map((i) => (
            <tr key={i.id} className="hover:bg-[var(--sb-hover)]">
              <Td className="font-medium">{i.title}</Td>
              <Td><Badge>{i.type_}</Badge></Td>
              <Td><Badge tone={i.severity === "critical" ? "red" : i.severity === "high" ? "orange" : "zinc"}>{i.severity}</Badge></Td>
              <Td><Badge tone={i.status === "resolved" ? "green" : "zinc"}>{i.status}</Badge></Td>
              <Td className="text-xs">{i.extra_cost > 0 ? i.extra_cost.toFixed(2) : "—"}</Td>
              <Td className="text-right">
                {i.status !== "resolved" && (
                  <Button variant="ghost" className="py-1 text-xs" onClick={() => void resolve(i.id)}>Resolve</Button>
                )}
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <IncidentForm
        open={open}
        onClose={() => setOpen(false)}
        trips={trips ?? []}
        tripId={tripId}
        setTripId={setTripId}
        onSubmit={createIncident}
      />
    </div>
  );
}

function IncidentForm({ open, onClose, trips, tripId, setTripId, onSubmit }: {
  open: boolean; onClose: () => void; trips: Trip[];
  tripId: string; setTripId: (v: string) => void;
  onSubmit: (b: { title: string; type_: string; severity?: string | null; description?: string | null; assigned_to?: string | null; extra_cost?: number | null }) => Promise<void> | void;
}) {
  const [title, setTitle] = useState(""); const [type_, setType_] = useState("issue");
  const [severity, setSeverity] = useState("medium"); const [desc, setDesc] = useState("");
  const [assigned, setAssigned] = useState(""); const [cost, setCost] = useState(0);
  const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setTitle(""); setType_("issue"); setSeverity("medium"); setDesc(""); setAssigned(""); setCost(0); setTripId(trips[0]?.id ?? ""); } }, [open, trips, setTripId]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ title, type_, severity: severity || null, description: desc || null, assigned_to: assigned || null, extra_cost: cost || null }); } finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="New incident" width={460}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Trip">
          <select value={tripId} onChange={(e) => setTripId(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {trips.map((t) => <option key={t.id} value={t.id}>{t.name}</option>)}
          </select>
        </Field>
        <Field label="Title"><Input required value={title} onChange={(e) => setTitle(e.target.value)} /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Type">
            <select value={type_} onChange={(e) => setType_(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["issue","alert","weather","medical","vehicle","guide","other"].map((t) => <option key={t} value={t}>{t}</option>)}
            </select>
          </Field>
          <Field label="Severity">
            <select value={severity} onChange={(e) => setSeverity(e.target.value)}
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["low","medium","high","critical"].map((s) => <option key={s} value={s}>{s}</option>)}
            </select>
          </Field>
        </div>
        <Field label="Description"><Textarea rows={2} value={desc} onChange={(e) => setDesc(e.target.value)} /></Field>
        <Field label="Assigned to"><Input value={assigned} onChange={(e) => setAssigned(e.target.value)} /></Field>
        <Field label="Extra cost (TZS)"><Input type="number" min={0} step={1} value={cost} onChange={(e) => setCost(Number(e.target.value))} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Create</Button>
        </div>
      </form>
    </SidePanel>
  );
}

// ============================================================ Trip Detail (linked from TripsPage) ===

type TripDetail = {
  trip: Trip & { start_date: string; end_date: string; pax_count: number; notes?: string | null };
  services: Array<{ id: string; kind: string; ref_id: string; day_number: number; unit_price: number; currency: string; qty: number; total: number }>;
  itineraries: Array<{ id: string; day_number: number; title: string; stops: Array<{ id: string; hour: string; activity?: string | null }> }>;
  destination_names: string[];
};

export function TripDetailPage() {
  const { id } = useParams<{ id: string }>();
  const [detail, setDetail] = useState<TripDetail | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [serviceOpen, setServiceOpen] = useState(false);

  useEffect(() => {
    if (!id) return;
    api.get<TripDetail>(`/dmc/trips/${id}/detail`)
      .then(setDetail)
      .catch((e) => setErr(e instanceof Error ? e.message : String(e)));
  }, [id]);

  async function reload() {
    if (!id) return;
    try { const d = await api.get<TripDetail>(`/dmc/trips/${id}/detail`); setDetail(d); }
    catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
  }

  async function addService(body: { kind: string; day_number: number; unit_price: number; currency: string; qty: number; total: number; notes?: string | null }) {
    await call(() => api.post(`/dmc/trips/${id}/services`, body));
    setServiceOpen(false); await reload();
  }

  async function delService(svcId: string) {
    if (!confirm("Remove this service?")) return;
    await call(() => api.del(`/dmc/trip-services/${svcId}`)); await reload();
  }

  const [docs, setDocs] = useState<Array<{ id: string; kind: string; filename: string; is_primary: boolean }>>([]);
  useEffect(() => { if (!id) return; api.get<{ documents: Array<{ id: string; kind: string; filename: string; is_primary: boolean }> }>(`/dmc/trips/${id}/documents`).then((r) => setDocs(r.documents)).catch(() => null); }, [id]);
  async function attachDoc(file: { id: number; filename: string; collection: string }) {
    await call(() => api.post(`/dmc/trips/${id}/documents`, { kind: "general", file_id: file.id, collection: file.collection, filename: file.filename }));
    const r = await api.get<{ documents: typeof docs }>(`/dmc/trips/${id}/documents`); setDocs(r.documents);
  }
  async function delDoc(docId: string) {
    if (!confirm("Remove this document?")) return;
    await call(() => api.del(`/dmc/documents/${docId}`)); const r = await api.get<{ documents: typeof docs }>(`/dmc/trips/${id}/documents`); setDocs(r.documents);
  }

  if (err) return <Alert>{err}</Alert>;
  if (!detail) return <Spinner />;
  const t = detail.trip;

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.name}
        description={`${t.start_date} → ${t.end_date} · ${t.pax_count} pax · ${t.status}`}
        actions={<Button onClick={() => setServiceOpen(true)}>Add service</Button>}
      />
      <div className="grid gap-4 sm:grid-cols-3">
        <Card className="p-4"><div className="text-xs text-[var(--app-fg-muted)]">Status</div><Badge tone={t.status === "confirmed" ? "green" : "zinc"}>{t.status}</Badge></Card>
        <Card className="p-4"><div className="text-xs text-[var(--app-fg-muted)]">Destinations</div><div className="text-sm font-medium">{detail.destination_names.join(", ") || "—"}</div></Card>
        <Card className="p-4"><div className="text-xs text-[var(--app-fg-muted)]">Services</div><div className="text-sm font-medium">{detail.services.length} items</div></Card>
      </div>
      {detail.itineraries.length > 0 && (
        <Card className="p-4 space-y-2">
          <h3 className="text-sm font-semibold">Itinerary</h3>
          {detail.itineraries.map((it) => (
            <div key={it.id} className="flex items-start gap-3 text-sm">
              <Badge>Day {it.day_number}</Badge>
              <div>
                <div className="font-medium">{it.title}</div>
                {it.stops.length > 0 && (
                  <div className="mt-1 text-xs text-[var(--app-fg-muted)]">
                    {it.stops.map((s) => `${s.hour} ${s.activity ?? ""}`).join(" · ")}
                  </div>
                )}
              </div>
            </div>
          ))}
        </Card>
      )}
      {detail.services.length > 0 && (
        <Card className="p-4 space-y-2">
          <h3 className="text-sm font-semibold">Services</h3>
          <Table head={["Kind", "Day", "Qty", "Unit Price", "Total", ""]}>
            {detail.services.map((s) => (
              <tr key={s.id} className="hover:bg-[var(--sb-hover)]">
                <Td><Badge>{s.kind}</Badge></Td>
                <Td>{s.day_number}</Td>
                <Td>{s.qty}</Td>
                <Td className="text-xs">{s.currency} {s.unit_price.toFixed(2)}</Td>
                <Td className="font-medium text-xs">{s.currency} {s.total.toFixed(2)}</Td>
                <Td className="text-right">
                  <Button variant="danger" className="py-1 text-xs" onClick={() => void delService(s.id)}>Delete</Button>
                </Td>
              </tr>
            ))}
          </Table>
        </Card>
      )}
      <Card className="p-4 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold">Documents</h3>
          <span className="text-xs text-[var(--app-fg-muted)]">{docs.length} file{docs.length !== 1 ? "s" : ""}</span>
        </div>
        {docs.length === 0 ? <p className="text-xs text-[var(--app-fg-muted)]">No documents attached.</p> : (
          <div className="space-y-1">
            {docs.map((d) => (
              <div key={d.id} className="flex items-center justify-between rounded-lg border border-[var(--app-border)] px-3 py-2">
                <div className="flex items-center gap-2">
                  <span className="text-xs">{d.filename}</span>
                  {d.is_primary && <Badge>Primary</Badge>}
                </div>
                <Button variant="danger" className="py-1 text-xs" onClick={() => void delDoc(d.id)}>Remove</Button>
              </div>
            ))}
          </div>
        )}
        <StoragePicker collection="dmc" onPick={(f) => void attachDoc(f)} />
      </Card>
      <ServiceForm open={serviceOpen} onClose={() => setServiceOpen(false)} onSubmit={addService} />
    </div>
  );
}

function ServiceForm({ open, onClose, onSubmit }: {
  open: boolean; onClose: () => void;
  onSubmit: (b: { kind: string; day_number: number; unit_price: number; currency: string; qty: number; total: number; notes?: string | null }) => Promise<void> | void;
}) {
  const [kind, setKind] = useState("activity");
  const [day, setDay] = useState(1); const [qty, setQty] = useState(1);
  const [price, setPrice] = useState(0); const [total, setTotal] = useState(0);
  const [currency, setCurrency] = useState("TZS"); const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setKind("activity"); setDay(1); setQty(1); setPrice(0); setTotal(0); setCurrency("TZS"); setNotes(""); } }, [open]);
  useEffect(() => { setTotal(price * qty); }, [price, qty]);
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true);
    try { await onSubmit({ kind, day_number: day, unit_price: price, currency, qty, total, notes: notes || null }); } finally { setBusy(false); }
  }
  return (
    <SidePanel open={open} onClose={onClose} title="Add service" width={440}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Kind">
          <select value={kind} onChange={(e) => setKind(e.target.value)}
            className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {["activity","accommodation","transport","guide","misc"].map((k) => <option key={k} value={k}>{k}</option>)}
          </select>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Day"><Input type="number" min={1} value={day} onChange={(e) => setDay(Number(e.target.value))} /></Field>
          <Field label="Qty"><Input type="number" min={1} value={qty} onChange={(e) => setQty(Number(e.target.value))} /></Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Unit price"><Input type="number" min={0} step={0.01} value={price} onChange={(e) => setPrice(Number(e.target.value))} /></Field>
          <Field label="Currency"><Input value={currency} onChange={(e) => setCurrency(e.target.value)} /></Field>
        </div>
        <div className="text-sm font-medium">Total: {currency} {total.toFixed(2)}</div>
        <Field label="Notes"><Textarea rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2">
          <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
          <Button type="submit" loading={busy}>Add service</Button>
        </div>
      </form>
    </SidePanel>
  );
}

export default DmcLayout;

// ============================================================ Wallet (PassKit) ===

type WalletPass = { pass_type_id: string; serial_number: string; updated_at: string };
type WalletIssued = { pass_type_id: string; serial_number: string; auth_token: string; download_url: string; qr_svg: string };
type WalletSample = { id: string; name: string; pass_style: string; description: string; colour: string };

export function WalletPage() {
  const [samples, setSamples] = useState<WalletSample[] | null>(null);
  const [passes, setPasses] = useState<WalletPass[] | null>(null);
  const [issued, setIssued] = useState<WalletIssued | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [selected, setSelected] = useState<WalletPass | null>(null);
  const [detail, setDetail] = useState<string>("");
  const [draft, setDraft] = useState<string>("");
  const [editorMode, setEditorMode] = useState<"visual" | "json">("visual");
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [panelError, setPanelError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function load() {
    try {
      const [s, p] = await Promise.all([
        api.get<{ samples: WalletSample[] }>("/dmc/wallet/samples"),
        api.get<{ passes: WalletPass[] }>("/dmc/wallet/passes"),
      ]);
      setSamples(s.samples); setPasses(p.passes);
    } catch (e) { setError(e instanceof Error ? e.message : "failed to load wallet"); }
  }
  useEffect(() => { void load(); }, []);

  async function issue(sample: WalletSample) {
    setBusyId(sample.id); setError(null);
    try {
      const res = await api.post<{ pass: WalletIssued }>("/dmc/wallet/issue", { sample_id: sample.id });
      setIssued(res.pass);
      const p = await api.get<{ passes: WalletPass[] }>("/dmc/wallet/passes"); setPasses(p.passes);
    } catch (e) { setError(e instanceof Error ? e.message : "issue failed"); }
    finally { setBusyId(null); }
  }

  async function openPass(row: WalletPass) {
    setSelected(row); setPanelError(null); setLoadingDetail(true); setDetail(""); setDraft("");
    try {
      const res = await api.get<{ pass: WalletPass; passJson: unknown }>(`/dmc/wallet/passes/${row.pass_type_id}/${row.serial_number}`);
      setDetail(JSON.stringify(res.passJson ?? {}, null, 2)); setDraft(JSON.stringify(res.passJson ?? {}, null, 2));
    } catch (e) { setPanelError(e instanceof Error ? e.message : "failed to load pass"); }
    finally { setLoadingDetail(false); }
  }

  const parsedPassData = useMemo<PKPassData | null>(() => { if (!draft) return null; try { return JSON.parse(draft) as PKPassData; } catch { return null; } }, [draft]);
  const dirty = useMemo(() => { if (!detail || !draft) return false; try { return JSON.stringify(JSON.parse(draft)) !== JSON.stringify(JSON.parse(detail)); } catch { return true; } }, [detail, draft]);

  function updateParsedData(mutator: (pass: PKPassData) => void) {
    if (!parsedPassData) return;
    const clone: PKPassData = JSON.parse(JSON.stringify(parsedPassData));
    mutator(clone); setDraft(JSON.stringify(clone, null, 2));
  }

  function savePass() {
    if (!selected) return;
    let body: unknown;
    try { body = JSON.parse(draft); } catch { setPanelError("invalid JSON"); return; }
    setSaving(true); setPanelError(null);
    api.patch(`/dmc/wallet/passes/${selected.pass_type_id}/${selected.serial_number}`, body)
      .then(() => { setDetail(JSON.stringify(body, null, 2)); void load(); })
      .catch((e) => setPanelError(e instanceof Error ? e.message : "save failed"))
      .finally(() => setSaving(false));
  }

  function deletePass() {
    if (!selected || !confirm(`Delete pass ${selected.serial_number}?`)) return;
    setSaving(true); setPanelError(null);
    api.del(`/dmc/wallet/passes/${selected.pass_type_id}/${selected.serial_number}`)
      .then(async () => { setIssued(null); await load(); setSelected(null); })
      .catch((e) => setPanelError(e instanceof Error ? e.message : "delete failed"))
      .finally(() => setSaving(false));
  }

  function removeRow(row: WalletPass) {
    if (!confirm(`Delete pass ${row.serial_number}?`)) return;
    setError(null);
    api.del(`/dmc/wallet/passes/${row.pass_type_id}/${row.serial_number}`)
      .then(async () => { if (selected?.serial_number === row.serial_number) setSelected(null); setIssued(null); await load(); })
      .catch((e) => setError(e instanceof Error ? e.message : "delete failed"));
  }

  return (
    <div className="space-y-8">
      <PageHeader title="Apple Wallet PassKit Designer" description="Issue signed .pkpass passes; scan QR with an iPhone on the same Wi-Fi." />
      {error && <Alert tone="error">{error}</Alert>}
      <section>
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">Issue from Sample Catalog</h2>
        {!samples ? <div className="grid place-items-center py-10"><Spinner /></div> : (
          <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
            {samples.map((s) => (
              <Card key={s.id} className="overflow-hidden">
                <div className="h-16 w-full" style={{ backgroundColor: s.colour }} aria-hidden />
                <div className="p-4">
                  <div className="flex items-center justify-between gap-2">
                    <h3 className="font-medium">{s.name}</h3><Badge>{s.pass_style}</Badge>
                  </div>
                  <p className="mt-1 text-xs text-[var(--app-fg-muted)]">{s.description}</p>
                  <Button variant="ghost" loading={busyId === s.id} className="mt-3 w-full" onClick={() => void issue(s)}>Issue Pass</Button>
                </div>
              </Card>
            ))}
          </div>
        )}
      </section>
      {issued && (
        <Card className="p-5">
          <div className="flex flex-wrap items-start justify-between gap-6">
            <div className="[&_svg]:rounded-lg [&_svg]:bg-white [&_svg]:p-2" dangerouslySetInnerHTML={{ __html: issued.qr_svg }} />
            <div className="min-w-64 flex-1 space-y-2 text-sm">
              <h3 className="font-semibold text-[var(--accent-fg)]">Pass Issued & Signed</h3>
              <dl className="space-y-1.5">
                <WalletRow label="Serial"><code className="font-mono text-xs">{issued.serial_number}</code></WalletRow>
                <WalletRow label="Auth Token"><code className="break-all font-mono text-xs">{issued.auth_token}</code></WalletRow>
                <WalletRow label="Download">
                  <a href={issued.download_url} className="text-sky-600 underline-offset-2 hover:underline dark:text-sky-300" target="_blank" rel="noreferrer">open .pkpass</a>
                </WalletRow>
              </dl>
              <Button variant="ghost" onClick={() => setIssued(null)}>Dismiss</Button>
            </div>
          </div>
        </Card>
      )}
      <section>
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">Issued Passes ({passes?.length ?? 0})</h2>
        {!passes ? <div className="grid place-items-center py-10"><Spinner /></div> : passes.length === 0 ? <EmptyState message="No passes issued yet." /> : (
          <Table head={["Serial", "Pass Type", "Updated", ""]}>
            {passes.map((row) => (
              <tr key={`${row.pass_type_id}/${row.serial_number}`} onClick={() => void openPass(row)}
                className={cn("cursor-pointer transition-colors", selected?.serial_number === row.serial_number && selected?.pass_type_id === row.pass_type_id ? "bg-[var(--accent-soft)]" : "hover:bg-[var(--sb-hover)]")}>
                <Td><code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 font-mono text-xs text-[var(--accent-fg)]">{row.serial_number}</code></Td>
                <Td className="text-[var(--app-fg-muted)]">{row.pass_type_id}</Td>
                <Td className="text-[var(--app-fg-muted)]">{new Date(row.updated_at + "Z").toLocaleString()}</Td>
                <Td className="text-right"><span className="flex items-center justify-end gap-2"><span className="text-xs text-[var(--app-fg-faint)]">design & edit ›</span><Button variant="danger" className="px-2.5 py-1 text-xs" onClick={(e) => { e.stopPropagation(); removeRow(row); }}>Delete</Button></span></Td>
              </tr>
            ))}
          </Table>
        )}
      </section>
      <SidePanel open={!!selected} onClose={() => setSelected(null)} title={`Designer: ${selected?.serial_number ?? ""}`}>
        {selected && (
          <>
            {panelError && <Alert tone="error">{panelError}</Alert>}
            <div className="flex gap-2 p-1 bg-[var(--app-card-2)] rounded-lg">
              <button type="button" className={cn("flex-1 py-1.5 text-xs font-medium rounded-md transition-all", editorMode === "visual" ? "bg-[var(--accent)] text-white shadow" : "text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]")} onClick={() => setEditorMode("visual")}>🎴 Visual Editor</button>
              <button type="button" className={cn("flex-1 py-1.5 text-xs font-medium rounded-md transition-all", editorMode === "json" ? "bg-[var(--accent)] text-white shadow" : "text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]")} onClick={() => setEditorMode("json")}>{`{ }`} Raw JSON</button>
            </div>
            {parsedPassData && (
              <PanelSection label="Live iOS Card Preview"><PassPreview passData={parsedPassData} /></PanelSection>
            )}
            {editorMode === "visual" && parsedPassData ? (
              <WalletVisualForm passData={parsedPassData} canWrite={!saving} onChange={updateParsedData} />
            ) : (
              <PanelSection label="Pass Body (pass.json)">
                {loadingDetail ? <div className="grid place-items-center py-6"><Spinner /></div> : (
                  <Textarea rows={14} spellCheck={false} value={draft} onChange={(e) => setDraft(e.target.value)} disabled={saving} />
                )}
              </PanelSection>
            )}
            <PanelSection label="Actions">
              <div className="flex gap-2">
                <Button className="flex-1" disabled={!dirty} loading={saving} onClick={savePass}>Save Changes</Button>
                <Button variant="ghost" className="flex-1" disabled={!dirty || saving} onClick={() => { setDraft(detail); setPanelError(null); }}>Reset</Button>
              </div>
              <Button variant="danger" className="mt-3 w-full" loading={saving} onClick={deletePass}>Delete Pass</Button>
            </PanelSection>
          </>
        )}
      </SidePanel>
    </div>
  );
}

function PanelSection({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">{label}</h3>
      {children}
    </div>
  );
}

function WalletRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-wrap items-baseline justify-between gap-2 border-b border-[var(--app-border)] pb-1.5">
      <dt className="text-xs uppercase tracking-wide text-[var(--app-fg-faint)]">{label}</dt>
      <dd>{children}</dd>
    </div>
  );
}

function WalletVisualForm({ passData, canWrite, onChange }: {
  passData: PKPassData; canWrite: boolean; onChange: (mutator: (pass: PKPassData) => void) => void;
}) {
  const { style, struct } = getPassStructure(passData);
  return (
    <div className="space-y-4">
      <PanelSection label="Pass Colors & Branding">
        <Field label="Logo Text"><Input value={passData.logoText || ""} disabled={!canWrite} onChange={(e) => onChange((p) => { p.logoText = e.target.value; })} placeholder="e.g. VIP Pass" /></Field>
        <div className="grid grid-cols-3 gap-2">
          <Field label="Background"><Input value={passData.backgroundColor || ""} disabled={!canWrite} onChange={(e) => onChange((p) => { p.backgroundColor = e.target.value; })} placeholder="rgb(30, 41, 59)" /></Field>
          <Field label="Foreground"><Input value={passData.foregroundColor || ""} disabled={!canWrite} onChange={(e) => onChange((p) => { p.foregroundColor = e.target.value; })} placeholder="rgb(255, 255, 255)" /></Field>
          <Field label="Labels"><Input value={passData.labelColor || ""} disabled={!canWrite} onChange={(e) => onChange((p) => { p.labelColor = e.target.value; })} placeholder="rgb(148, 163, 184)" /></Field>
        </div>
      </PanelSection>
      <PanelSection label={`Primary Fields (${style})`}>
        {(struct.primaryFields ?? []).map((field, idx) => (
          <div key={idx} className="flex items-center gap-2 p-2 border border-[var(--app-border)] rounded-lg">
            <div className="flex-1 space-y-1">
              <Input value={field.label || ""} disabled={!canWrite} placeholder="Label" onChange={(e) => onChange((p) => { const { struct: st } = getPassStructure(p); if (st.primaryFields?.[idx]) st.primaryFields![idx].label = e.target.value; })} />
              <Input value={field.value ?? ""} disabled={!canWrite} placeholder="Value" onChange={(e) => onChange((p) => { const { struct: st } = getPassStructure(p); if (st.primaryFields?.[idx]) st.primaryFields![idx].value = e.target.value; })} />
            </div>
            {canWrite && <Button variant="danger" className="px-2 py-1 text-xs" onClick={() => onChange((p) => { const { struct: st } = getPassStructure(p); st.primaryFields?.splice(idx, 1); })}>✕</Button>}
          </div>
        ))}
        {canWrite && <Button variant="ghost" className="w-full text-xs" onClick={() => onChange((p) => { const { struct: st } = getPassStructure(p); if (!st.primaryFields) st.primaryFields = []; st.primaryFields.push({ key: `primary_${Date.now()}`, label: "LABEL", value: "VALUE" }); })}>+ Add Primary Field</Button>}
      </PanelSection>
      <PanelSection label="Secondary Fields">
        {(struct.secondaryFields ?? []).map((field, idx) => (
          <div key={idx} className="flex items-center gap-2 p-2 border border-[var(--app-border)] rounded-lg">
            <div className="flex-1 space-y-1">
              <Input value={field.label || ""} disabled={!canWrite} placeholder="Label" onChange={(e) => onChange((p) => { const { struct: st } = getPassStructure(p); if (st.secondaryFields?.[idx]) st.secondaryFields![idx].label = e.target.value; })} />
              <Input value={field.value ?? ""} disabled={!canWrite} placeholder="Value" onChange={(e) => onChange((p) => { const { struct: st } = getPassStructure(p); if (st.secondaryFields?.[idx]) st.secondaryFields![idx].value = e.target.value; })} />
            </div>
            {canWrite && <Button variant="danger" className="px-2 py-1 text-xs" onClick={() => onChange((p) => { const { struct: st } = getPassStructure(p); st.secondaryFields?.splice(idx, 1); })}>✕</Button>}
          </div>
        ))}
        {canWrite && <Button variant="ghost" className="w-full text-xs" onClick={() => onChange((p) => { const { struct: st } = getPassStructure(p); if (!st.secondaryFields) st.secondaryFields = []; st.secondaryFields.push({ key: `sec_${Date.now()}`, label: "LABEL", value: "VALUE" }); })}>+ Add Secondary Field</Button>}
      </PanelSection>
    </div>
  );
}
