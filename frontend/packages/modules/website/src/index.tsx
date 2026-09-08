// Website module — Template 0 (Knowlia download-hub port) + sub-modules:
// hero, slideshow, downloads, products, cart, bookings, links, theme, preview
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

const cn = (...parts: Array<string | false | null | undefined>) => parts.filter(Boolean).join(" ");

type SubNav = { to: string; label: string; desc: string };
export const WEBSITE_NAV: SubNav[] = [
  { to: "/website/hero", label: "Hero", desc: "Title · tagline · badge" },
  { to: "/website/slideshow", label: "Slideshow", desc: "Gallery" },
  { to: "/website/downloads", label: "Downloads", desc: "Platforms · links" },
  { to: "/website/links", label: "Links", desc: "Navigation" },
  { to: "/website/products", label: "Products", desc: "Catalog showcase" },
  { to: "/website/cart", label: "Cart", desc: "Cart · checkout" },
  { to: "/website/bookings", label: "Bookings", desc: "Orders · appointments" },
  { to: "/website/theme", label: "Theme", desc: "Colors · fonts" },
  { to: "/website/preview", label: "Preview", desc: "Template 0 live" },
];

// ----------------------------------------------------- layout / shell --

export function WebsiteLayout() {
  const loc = useLocation();
  return (
    <div className="flex gap-6" style={{ marginLeft: 0 }}>
      <aside
        className="hidden w-[210px] shrink-0 lg:block"
        style={{ position: "sticky", top: 80, alignSelf: "flex-start", height: "calc(100dvh - 96px)" }}
      >
        <div className="rounded-xl border border-[var(--app-border)] bg-[var(--app-card)] p-3">
          <div className="px-2 pb-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            Website · Template 0
          </div>
          <nav className="flex flex-col gap-0.5">
            {WEBSITE_NAV.map((i) => (
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
          {WEBSITE_NAV.map((i) => (
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
        {loc.pathname === "/website" && <WebsiteOverview />}
      </div>
    </div>
  );
}

export function WebsiteOverview() {
  const nav = useNavigate();
  return (
    <div className="space-y-6">
      <PageHeader title="Website" description="Template 0 — Knowlia download-hub → products → cart → bookings. Edit each section below." />
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {WEBSITE_NAV.map((i) => (
          <Card key={i.to} className="cursor-pointer p-5 transition-shadow hover:shadow-md" onClick={() => nav(i.to)}>
            <div className="text-sm font-semibold">{i.label}</div>
            <div className="mt-1 text-xs text-[var(--app-fg-muted)]">{i.desc}</div>
            <div className="mt-3 text-xs text-[var(--accent-fg)]">Open →</div>
          </Card>
        ))}
      </div>
      <Card className="p-5">
        <div className="text-sm font-semibold">Template 0 — Knowlia Dark</div>
        <p className="mt-1 text-sm text-[var(--app-fg-muted)]">
          Ported from <code className="rounded bg-[var(--accent-soft)] px-1 py-0.5 text-xs">H:/Github/download-hub/src/pages/Index.tsx</code> — dark <code>#1a1a1a</code> grid + hero + download cards.
        </p>
        <div className="mt-3">
          <Button variant="ghost" onClick={() => nav("/website/preview")}>Live preview →</Button>
        </div>
      </Card>
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
  useEffect(() => { void load(); }, [url, key]);
  return { data, err, reload: load };
}
async function call<T>(fn: () => Promise<T>): Promise<T> {
  try { return await fn(); } catch (e) { alert(e instanceof Error ? e.message : String(e)); throw e; }
}

// ============================================================================
// Hero sub-module
// ============================================================================

type WebsiteSection = { id: string; template_id: string; kind: string; position: number; config_json: string; is_visible: boolean; updated_at?: string };
type Template = { id: string; slug: string; name: string; version: string; is_active: boolean; theme_json?: string | null };

export function HeroPage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const [hero, setHero] = useState<WebsiteSection | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  // form state
  const [title, setTitle] = useState("K N O W L I A");
  const [subtitle, setSubtitle] = useState("KNOWLEDGE + UTOPIA");
  const [badge, setBadge] = useState("Version 1.1.1");
  const [tagline, setTagline] = useState("Welcome to Knowlia: where knowledge meets innovation. Experience streamlined course management, intuitive real-time campus navigation, and stress-free organization—all crafted exclusively for COICT students.");
  const [visible, setVisible] = useState(true);

  async function loadHero() {
    if (!tpl) return;
    try {
      const r = await api.get<{ section: WebsiteSection }>(`/website/templates/${tpl.id}/sections/hero`);
      setHero(r.section);
      try {
        const cfg = JSON.parse(r.section.config_json || "{}");
        if (cfg.title) setTitle(cfg.title);
        if (cfg.subtitle) setSubtitle(cfg.subtitle);
        if (cfg.badge) setBadge(cfg.badge);
        if (cfg.tagline) setTagline(cfg.tagline);
        setVisible(r.section.is_visible);
      } catch {}
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => { if (tpl) void loadHero(); }, [tpl?.id]);

  async function save(e: React.FormEvent) {
    e.preventDefault();
    if (!tpl) return;
    setSaving(true);
    try {
      await call(() => api.patch<WebsiteSection>(`/website/templates/${tpl.id}/sections/hero`, {
        config_json: JSON.stringify({ title, subtitle, badge, tagline }),
        is_visible: visible,
      }));
      await loadHero();
    } finally { setSaving(false); }
  }

  // also allow init if no hero row exists
  async function init() {
    if (!tpl) return;
    await call(() => api.post(`/website/templates/${tpl.id}/sections`, { kind: "hero", position: 0, config_json: JSON.stringify({ title, subtitle, badge, tagline }), is_visible: visible }));
    await loadHero();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Hero" description="Top banner of Template 0 — mirrors download-hub hero index." actions={<Badge tone="zinc">{tpl ? tpl.slug : "loading"}</Badge>} />
      {err && <Alert>{err}</Alert>}
      {/* live mini preview */}
      <Card className="overflow-hidden">
        <div className="relative bg-[#1a1a1a] p-8 text-center" style={{ backgroundImage: "linear-gradient(to right,#fafafa0f 1px,transparent 1px),linear-gradient(to bottom,#fafafa0f 1px,transparent 1px)", backgroundSize: "20px 20px" }}>
          <h1 className="text-4xl font-bold tracking-tight text-[#fafafa]" style={{ fontFamily: "luckiest-guy-regular, cursive" }}>{title}</h1>
          <p className="mt-1 text-xl text-[#bcbcbc] opacity-60" style={{ fontFamily: "sister-spray, cursive" }}>{subtitle}</p>
          <span className="mt-2 inline-block rounded-full bg-[#3a6ea4] px-3 py-1 text-xs text-white">{badge}</span>
          <p className="mx-auto mt-4 max-w-2xl text-sm leading-relaxed text-gray-300">{tagline}</p>
          {!visible && <div className="mt-3 text-xs text-red-300"> Hidden </div>}
        </div>
      </Card>

      <form onSubmit={save} className="space-y-4">
        <Field label="Title (hero h1)"><Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="K N O W L I A" /></Field>
        <Field label="Subtitle"><Input value={subtitle} onChange={(e) => setSubtitle(e.target.value)} /></Field>
        <Field label="Badge"><Input value={badge} onChange={(e) => setBadge(e.target.value)} placeholder="Version 1.1.1" /></Field>
        <Field label="Tagline"><Textarea rows={3} value={tagline} onChange={(e) => setTagline(e.target.value)} /></Field>
        <label className="flex items-center gap-2 text-sm"><input type="checkbox" checked={visible} onChange={(e) => setVisible(e.target.checked)} /> Visible</label>
        <div className="flex gap-2">
          <Button type="submit" loading={saving}>Save hero</Button>
          {hero === null && <Button type="button" variant="ghost" onClick={init}>Initialize hero section</Button>}
        </div>
      </form>
      {!tpl && <Alert tone="info">No template yet — create Template 0 first from the API or run the website migration seed.</Alert>}
    </div>
  );
}

// ============================================================================
// Slideshow
// ============================================================================

type Slide = { url: string; alt?: string };

export function SlideshowPage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const [section, setSection] = useState<WebsiteSection | null>(null);
  const [slides, setSlides] = useState<Slide[]>([{ url: "https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=1200" }]);
  const [idx, setIdx] = useState(0);
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function load() {
    if (!tpl) return;
    try {
      const r = await api.get<{ section: WebsiteSection }>(`/website/templates/${tpl.id}/sections/slideshow`);
      setSection(r.section);
      const cfg = JSON.parse(r.section.config_json || "{}");
      if (Array.isArray(cfg.slides) && cfg.slides.length) setSlides(cfg.slides);
    } catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
  }
  useEffect(() => { if (tpl) void load(); }, [tpl?.id]);
  async function save() {
    if (!tpl) return;
    setSaving(true);
    try { await call(() => api.patch(`/website/templates/${tpl.id}/sections/slideshow`, { config_json: JSON.stringify({ slides }) })); await load(); } finally { setSaving(false); }
  }
  async function ensure() {
    if (!tpl) return;
    await call(() => api.post(`/website/templates/${tpl.id}/sections`, { kind: "slideshow", position: 1, config_json: JSON.stringify({ slides }), is_visible: true }));
    await load();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Slideshow" description="Ported from AppSlideshow — images carousel." actions={<Button onClick={save} loading={saving}>Save slides</Button>} />
      {err && <Alert>{err} {!section && <button className="ml-2 underline" onClick={ensure}>Create slides section</button>}</Alert>}
      <Card className="overflow-hidden">
        <div className="relative aspect-[16/9] overflow-hidden bg-black">
          {slides[idx] && <img src={slides[idx].url} alt={slides[idx].alt ?? `Slide ${idx + 1}`} className="h-full w-full object-contain" />}
          <button onClick={() => setIdx((i) => (i === 0 ? slides.length - 1 : i - 1))} className="absolute left-3 top-1/2 -translate-y-1/2 rounded-full bg-white/10 px-3 py-2 text-white backdrop-blur">‹</button>
          <button onClick={() => setIdx((i) => (i === slides.length - 1 ? 0 : i + 1))} className="absolute right-3 top-1/2 -translate-y-1/2 rounded-full bg-white/10 px-3 py-2 text-white backdrop-blur">›</button>
          <div className="absolute bottom-2 left-1/2 flex -translate-x-1/2 gap-1.5">
            {slides.map((_, i) => (<span key={i} className={cn("h-1.5 rounded-full transition-all", i === idx ? "w-6 bg-blue-500" : "w-2 bg-white/40")} />))}
          </div>
        </div>
      </Card>
      <div className="space-y-2">
        {slides.map((s, i) => (
          <div key={i} className="flex gap-2">
            <Input className="flex-1" value={s.url} onChange={(e) => setSlides((a) => a.map((x, j) => j === i ? { ...x, url: e.target.value } : x))} placeholder="https://..." />
            <Button variant="ghost" onClick={() => setSlides((a) => a.filter((_, j) => j !== i))}>Remove</Button>
          </div>
        ))}
        <Button variant="ghost" onClick={() => setSlides((a) => [...a, { url: "" }])}>+ Add slide</Button>
      </div>
    </div>
  );
}

// ============================================================================
// Downloads (platforms + versions)
// ============================================================================

type DownloadVersion = { arch: string; url: string; recommended?: boolean };
type DownloadPlatform = { platform: string; versions: DownloadVersion[] };

export function DownloadsPage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const [platforms, setPlatforms] = useState<DownloadPlatform[]>([
    { platform: "Windows", versions: [{ arch: "64-bit", url: "/app/builds/build-windows-x64/Knowlia_1.1.1_x64_en-US.msi", recommended: true }, { arch: "32-bit", url: "/app/builds/build-windows-x86/Knowlia_1.1.1_x86_en-US.msi", recommended: false }] },
  ]);
  const [err, setErr] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function load() {
    if (!tpl) return;
    try {
      const r = await api.get<{ section: WebsiteSection }>(`/website/templates/${tpl.id}/sections/downloads`);
      const cfg = JSON.parse(r.section.config_json || "{}");
      if (Array.isArray(cfg.platforms)) setPlatforms(cfg.platforms);
    } catch (e) { setErr(e instanceof Error ? e.message : String(e)); }
  }
  useEffect(() => { if (tpl) void load(); }, [tpl?.id]);
  async function save() {
    if (!tpl) return;
    setSaving(true);
    try {
      // upsert via POST if not exists, fallback to PATCH
      try {
        await call(() => api.post(`/website/templates/${tpl.id}/sections`, { kind: "downloads", position: 2, config_json: JSON.stringify({ platforms }), is_visible: true }));
      } catch {}
      await call(() => api.patch(`/website/templates/${tpl.id}/sections/downloads`, { config_json: JSON.stringify({ platforms }) }));
      await load();
    } finally { setSaving(false); }
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Downloads" description="Download platforms (replaces DownloadCard grid)." actions={<Button onClick={save} loading={saving}>Save</Button>} />
      {err && <Alert tone="info">{err}</Alert>}
      <div className="space-y-4">
        {platforms.map((p, pi) => (
          <Card key={pi} className="p-4">
            <div className="flex items-center gap-2">
              <Input className="flex-1" value={p.platform} onChange={(e) => setPlatforms((a) => a.map((x, i) => i === pi ? { ...x, platform: e.target.value } : x))} placeholder="Platform" />
              <Button variant="ghost" onClick={() => setPlatforms((a) => a.filter((_, i) => i !== pi))}>Remove platform</Button>
            </div>
            <div className="mt-3 space-y-2">
              {p.versions.map((v, vi) => (
                <div key={vi} className="flex gap-2">
                  <Input placeholder="arch" value={v.arch} onChange={(e) => setPlatforms((a) => a.map((x, i) => i === pi ? { ...x, versions: x.versions.map((vv, j) => j === vi ? { ...vv, arch: e.target.value } : vv) } : x))} />
                  <Input className="flex-[2]" placeholder="url" value={v.url} onChange={(e) => setPlatforms((a) => a.map((x, i) => i === pi ? { ...x, versions: x.versions.map((vv, j) => j === vi ? { ...vv, url: e.target.value } : vv) } : x))} />
                  <label className="flex items-center gap-1 text-xs whitespace-nowrap"><input type="checkbox" checked={!!v.recommended} onChange={(e) => setPlatforms((a) => a.map((x, i) => i === pi ? { ...x, versions: x.versions.map((vv, j) => j === vi ? { ...vv, recommended: e.target.checked } : vv) } : x))} /> rec</label>
                  <Button variant="ghost" className="px-2 py-1 text-xs" onClick={() => setPlatforms((a) => a.map((x, i) => i === pi ? { ...x, versions: x.versions.filter((_, j) => j !== vi) } : x))}>×</Button>
                </div>
              ))}
              <Button variant="ghost" className="text-xs" onClick={() => setPlatforms((a) => a.map((x, i) => i === pi ? { ...x, versions: [...x.versions, { arch: "", url: "" }] } : x))}>+ Add version</Button>
            </div>
          </Card>
        ))}
        <Button variant="ghost" onClick={() => setPlatforms((a) => [...a, { platform: "", versions: [{ arch: "", url: "" }] }])}>+ Add platform</Button>
      </div>
      <div className="grid gap-4 md:grid-cols-2">
        {platforms.map((p) => (
          <Card key={p.platform} className="p-4">
            <div className="text-sm font-semibold">{p.platform || "Untitled"}</div>
            <div className="mt-2 space-y-1">
              {p.versions.map((v) => (
                <div key={v.arch + v.url} className="flex items-center justify-between rounded border px-2 py-1.5 text-xs">
                  <span>{v.arch} {v.recommended && <Badge tone="green">recommended</Badge>}</span>
                  <a href={v.url} target="_blank" rel="noreferrer" className="text-[var(--accent)] underline">download</a>
                </div>
              ))}
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// Links (site navigation)
// ============================================================================

type WebsiteLink = { id: string; template_id: string; label: string; href: string; icon?: string | null; position: number; is_visible: boolean };

export function LinksPage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const { data, err, reload } = useList<WebsiteLink>(tpl ? `/website/templates/${tpl.id}/links` : "/website/templates/__none__/links", "links");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<WebsiteLink | null>(null);
  async function create(body: Partial<WebsiteLink>) {
    if (!tpl) return;
    await call(() => api.post<WebsiteLink>(`/website/templates/${tpl.id}/links`, { label: "", href: "", position: (data?.length ?? 0), is_visible: true, ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<WebsiteLink>) {
    if (!tpl) return;
    await call(() => api.patch<WebsiteLink>(`/website/templates/${tpl.id}/links/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!tpl) return;
    if (!confirm("Delete link?")) return;
    await call(() => api.del<void>(`/website/templates/${tpl.id}/links/${id}`));
    await reload();
  }
  return (
    <div className="space-y-6">
      <PageHeader title="Links" description="Top navigation / footer links for the website." actions={<Button onClick={() => setOpen(true)} disabled={!tpl}>New link</Button>} />
      {err && <Alert>{err}</Alert>}
      {!tpl && <Alert tone="info">No template yet.</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No links yet." /> : (
        <Table head={["Label", "Href", "Position", "Visible", ""]}>
          {data.map((l) => (
            <tr key={l.id}>
              <Td className="font-medium">{l.label}</Td>
              <Td className="font-mono text-xs"><a href={l.href} target="_blank" rel="noreferrer" className="text-[var(--accent)] underline">{l.href}</a></Td>
              <Td>{l.position}</Td>
              <Td><Badge tone={l.is_visible ? "green" : "zinc"}>{String(l.is_visible)}</Badge></Td>
              <Td className="space-x-2 text-right">
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(l)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(l.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <LinkForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New link" />
      <LinkForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)} onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.label ?? ""}`} />
    </div>
  );
}

function LinkForm({ open, initial, onClose, onSubmit, title }: { open: boolean; initial?: WebsiteLink; onClose: () => void; onSubmit: (b: Partial<WebsiteLink>) => Promise<void> | void; title: string }) {
  const [label, setLabel] = useState(initial?.label ?? "");
  const [href, setHref] = useState(initial?.href ?? "");
  const [icon, setIcon] = useState(initial?.icon ?? "");
  const [pos, setPos] = useState(initial?.position ?? 0);
  const [visible, setVisible] = useState(initial?.is_visible ?? true);
  const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setLabel(initial?.label ?? ""); setHref(initial?.href ?? ""); setIcon(initial?.icon ?? ""); setPos(initial?.position ?? 0); setVisible(initial?.is_visible ?? true); } }, [open, initial]);
  async function submit(e: React.FormEvent) { e.preventDefault(); setBusy(true); try { await onSubmit({ label, href, icon: icon || null, position: pos, is_visible: visible }); } finally { setBusy(false); } }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={420}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Label"><Input required value={label} onChange={(e) => setLabel(e.target.value)} placeholder="Home / Products / Contact" /></Field>
        <Field label="Href"><Input required value={href} onChange={(e) => setHref(e.target.value)} placeholder="/ or https://..." /></Field>
        <Field label="Icon (optional)"><Input value={icon} onChange={(e) => setIcon(e.target.value)} placeholder="hero icon name" /></Field>
        <Field label="Position"><Input type="number" value={pos} onChange={(e) => setPos(Number(e.target.value))} /></Field>
        <label className="flex items-center gap-2 text-sm"><input type="checkbox" checked={visible} onChange={(e) => setVisible(e.target.checked)} /> Visible</label>
        <div className="flex justify-end gap-2 pt-2"><Button type="button" variant="ghost" onClick={onClose}>Cancel</Button><Button type="submit" loading={busy}>Save</Button></div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Products showcase (read from ERP)
// ============================================================================

type Product = { id: string; sku: string; name: string; description?: string | null; is_active: boolean };

export function ProductsPage() {
  const { data, err } = useList<Product>("/erp/products", "products");
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? null;
  const [featuredIds, setFeaturedIds] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);

  async function loadFeatured() {
    if (!tpl) return;
    try {
      const r = await api.get<{ section: WebsiteSection }>(`/website/templates/${tpl.id}/sections/products`);
      const cfg = JSON.parse(r.section.config_json || "{}");
      if (Array.isArray(cfg.featured_product_ids)) setFeaturedIds(cfg.featured_product_ids);
    } catch {}
  }
  useEffect(() => { if (tpl) void loadFeatured(); }, [tpl?.id]);
  async function saveFeatured() {
    if (!tpl) return; setSaving(true);
    try {
      try { await api.post(`/website/templates/${tpl.id}/sections`, { kind: "products", position: 3, config_json: JSON.stringify({ featured_product_ids: featuredIds }), is_visible: true }); } catch {}
      await call(() => api.patch(`/website/templates/${tpl.id}/sections/products`, { config_json: JSON.stringify({ featured_product_ids: featuredIds }) }));
      await loadFeatured();
    } finally { setSaving(false); }
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Products" description="Showcase — pick featured products from ERP catalog to display on the website." actions={<Button onClick={saveFeatured} loading={saving} disabled={!tpl}>Save featured</Button>} />
      {err && <Alert>{err}</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No ERP products yet — create some in ERP → Products." /> : (
        <Table head={["SKU", "Name", "Active", "Featured", ""]}>
          {data.map((p) => {
            const isFeatured = featuredIds.includes(p.id);
            return (
              <tr key={p.id}>
                <Td><code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 text-xs">{p.sku}</code></Td>
                <Td className="font-medium">{p.name}</Td>
                <Td><Badge tone={p.is_active ? "green" : "zinc"}>{String(p.is_active)}</Badge></Td>
                <Td><Badge tone={isFeatured ? "green" : "zinc"}>{isFeatured ? "yes" : "no"}</Badge></Td>
                <Td className="text-right">
                  <Button variant="ghost" className="py-1 text-xs" onClick={() => setFeaturedIds((a) => isFeatured ? a.filter((x) => x !== p.id) : [...a, p.id])}>
                    {isFeatured ? "Unfeature" : "Feature"}
                  </Button>
                </Td>
              </tr>
            );
          })}
        </Table>
      )}
      <Card className="p-4">
        <div className="text-sm font-medium">Featured product showcase preview</div>
        <div className="mt-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {(data ?? []).filter((p) => featuredIds.includes(p.id)).map((p) => (
            <Card key={p.id} className="p-4">
              <div className="text-sm font-semibold">{p.name}</div>
              <div className="mt-1 text-xs text-[var(--app-fg-muted)]">{p.sku} · {p.description ?? "—"}</div>
              <div className="mt-3"><Button variant="ghost" className="text-xs" onClick={() => setFeaturedIds((a) => a.filter((x) => x !== p.id))}>Remove</Button></div>
            </Card>
          ))}
          {featuredIds.length === 0 && <div className="text-sm text-[var(--app-fg-muted)]">No featured products selected.</div>}
        </div>
      </Card>
    </div>
  );
}

// ============================================================================
// Cart
// ============================================================================

type CartItem = { id: string; template_id: string; user_id: string; product_id: string; quantity: number; unit_price: number; added_at?: string };
type CartRow = CartItem & { product_name?: string; sku?: string };

export function CartPage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const { data, err, reload } = useList<CartRow>(tpl ? `/website/templates/${tpl.id}/cart` : "/website/templates/__none__/cart", "items");
  const products = useList<Product>("/erp/products", "products");
  const [adding, setAdding] = useState(false);

  async function add(productId: string, qty: number) {
    if (!tpl) return;
    await call(() => api.post(`/website/templates/${tpl.id}/cart`, { product_id: productId, quantity: qty }));
    await reload();
  }
  async function remove(id: string) {
    if (!tpl) return;
    await call(() => api.del<void>(`/website/templates/${tpl.id}/cart/${id}`));
    await reload();
  }
  async function checkout() {
    if (!tpl) return;
    const res = await call(() => api.post<{ order_id: string }>(`/website/templates/${tpl.id}/cart/checkout`, {}));
    alert(`Checkout OK — order ${res.order_id}`);
    await reload();
  }
  const total = (data ?? []).reduce((s, r) => s + (r.quantity * (r.unit_price || 0)), 0);

  return (
    <div className="space-y-6">
      <PageHeader title="Cart" description="Customer cart for this template — add products, adjust, checkout to sales order." actions={<div className="flex gap-2"><Button variant="ghost" onClick={() => setAdding(true)} disabled={!tpl}>Add item</Button><Button onClick={checkout} disabled={!data || data.length === 0}>Checkout</Button></div>} />
      {err && <Alert>{err}</Alert>}
      {!tpl && <Alert tone="info">No template yet.</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="Cart is empty." /> : (
        <>
          <Table head={["Product", "Qty", "Unit price", "Total", ""]}>
            {data.map((r) => (
              <tr key={r.id}>
                <Td className="font-medium">{r.product_name ?? r.product_id.slice(0, 8)} <span className="text-xs text-[var(--app-fg-muted)]">{r.sku ?? ""}</span></Td>
                <Td>{r.quantity}</Td>
                <Td>${(r.unit_price || 0).toFixed(2)}</Td>
                <Td>${((r.quantity * (r.unit_price || 0))).toFixed(2)}</Td>
                <Td className="text-right"><Button variant="danger" className="py-1 text-xs" onClick={() => remove(r.id)}>Remove</Button></Td>
              </tr>
            ))}
          </Table>
          <Card className="p-4 text-right"><span className="text-sm font-semibold">Total: ${total.toFixed(2)}</span></Card>
        </>
      )}
      <AddCartPanel open={adding} onClose={() => setAdding(false)} products={products.data ?? []} onAdd={add} />
    </div>
  );
}

function AddCartPanel({ open, onClose, products, onAdd }: { open: boolean; onClose: () => void; products: { id: string; sku: string; name: string }[]; onAdd: (pid: string, qty: number) => Promise<void> | void }) {
  const [pid, setPid] = useState("");
  const [qty, setQty] = useState(1);
  const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setPid(products[0]?.id ?? ""); setQty(1); } }, [open, products]);
  async function submit(e: React.FormEvent) { e.preventDefault(); if (!pid) return; setBusy(true); try { await onAdd(pid, qty); onClose(); } finally { setBusy(false); } }
  return (
    <SidePanel open={open} onClose={onClose} title="Add to cart" width={360}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Product">
          <select required value={pid} onChange={(e) => setPid(e.target.value)} className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
            {products.map((p) => (<option key={p.id} value={p.id}>{p.sku} — {p.name}</option>))}
          </select>
        </Field>
        <Field label="Quantity"><Input type="number" min={1} step={1} value={qty} onChange={(e) => setQty(Number(e.target.value))} /></Field>
        <div className="flex justify-end gap-2 pt-2"><Button type="button" variant="ghost" onClick={onClose}>Cancel</Button><Button type="submit" loading={busy}>Add</Button></div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Bookings
// ============================================================================

type Booking = { id: string; template_id: string; customer_name: string; customer_email?: string | null; service?: string | null; product_id?: string | null; date: string; status: string; notes?: string | null };

export function BookingsPage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const { data, err, reload } = useList<Booking>(tpl ? `/website/templates/${tpl.id}/bookings` : "/website/templates/__none__/bookings", "bookings");
  const [open, setOpen] = useState(false);
  const [edit, setEdit] = useState<Booking | null>(null);

  async function create(body: Partial<Booking>) {
    if (!tpl) return;
    await call(() => api.post<Booking>(`/website/templates/${tpl.id}/bookings`, { status: "PENDING", date: new Date().toISOString().slice(0, 10), ...body }));
    setOpen(false); await reload();
  }
  async function update(id: string, body: Partial<Booking>) {
    if (!tpl) return;
    await call(() => api.patch<Booking>(`/website/templates/${tpl.id}/bookings/${id}`, body));
    setEdit(null); await reload();
  }
  async function del(id: string) {
    if (!tpl) return;
    if (!confirm("Delete booking?")) return;
    await call(() => api.del<void>(`/website/templates/${tpl.id}/bookings/${id}`));
    await reload();
  }
  async function confirmBooking(id: string) {
    if (!tpl) return;
    await call(() => api.post(`/website/templates/${tpl.id}/bookings/${id}/confirm`, {}));
    await reload();
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Bookings" description="Orders / appointments for this website." actions={<Button onClick={() => setOpen(true)} disabled={!tpl}>New booking</Button>} />
      {err && <Alert>{err}</Alert>}
      {!tpl && <Alert tone="info">No template yet.</Alert>}
      {!data ? <Spinner /> : data.length === 0 ? <EmptyState message="No bookings yet." /> : (
        <Table head={["Customer", "Service", "Date", "Status", ""]}>
          {data.map((b) => (
            <tr key={b.id}>
              <Td className="font-medium">{b.customer_name} <span className="block text-xs text-[var(--app-fg-muted)]">{b.customer_email ?? "—"}</span></Td>
              <Td className="text-sm">{b.service ?? (b.product_id ? b.product_id.slice(0, 8) : "—")}</Td>
              <Td className="text-xs">{new Date(b.date).toLocaleDateString()}</Td>
              <Td><Badge tone={b.status === "CONFIRMED" ? "green" : b.status === "CANCELLED" ? "red" : "zinc"}>{b.status}</Badge></Td>
              <Td className="space-x-2 text-right">
                {b.status === "PENDING" && <Button variant="ghost" className="py-1 text-xs" onClick={() => confirmBooking(b.id)}>Confirm</Button>}
                <Button variant="ghost" className="py-1 text-xs" onClick={() => setEdit(b)}>Edit</Button>
                <Button variant="danger" className="py-1 text-xs" onClick={() => del(b.id)}>Delete</Button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
      <BookingForm open={open} onClose={() => setOpen(false)} onSubmit={create} title="New booking" />
      <BookingForm open={!!edit} initial={edit ?? undefined} onClose={() => setEdit(null)} onSubmit={async (b) => { if (edit) await update(edit.id, b); }} title={`Edit ${edit?.customer_name ?? ""}`} />
    </div>
  );
}

function BookingForm({ open, initial, onClose, onSubmit, title }: { open: boolean; initial?: Booking; onClose: () => void; onSubmit: (b: Partial<Booking>) => Promise<void> | void; title: string }) {
  const [customerName, setCustomerName] = useState("");
  const [email, setEmail] = useState("");
  const [service, setService] = useState("");
  const [date, setDate] = useState(new Date().toISOString().slice(0, 10));
  const [status, setStatus] = useState("PENDING");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => { if (open) { setCustomerName(initial?.customer_name ?? ""); setEmail(initial?.customer_email ?? ""); setService(initial?.service ?? ""); setDate(initial?.date ? initial.date.slice(0, 10) : new Date().toISOString().slice(0, 10)); setStatus(initial?.status ?? "PENDING"); setNotes(initial?.notes ?? ""); } }, [open, initial]);
  async function submit(e: React.FormEvent) { e.preventDefault(); setBusy(true); try { await onSubmit({ customer_name: customerName, customer_email: email || null, service: service || null, date: new Date(date).toISOString(), status, notes: notes || null }); } finally { setBusy(false); } }
  return (
    <SidePanel open={open} onClose={onClose} title={title} width={440}>
      <form onSubmit={submit} className="space-y-4">
        <Field label="Customer name"><Input required value={customerName} onChange={(e) => setCustomerName(e.target.value)} /></Field>
        <Field label="Customer email"><Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} /></Field>
        <Field label="Service / Product"><Input value={service} onChange={(e) => setService(e.target.value)} placeholder="Consulting Hour, Laptop Pro ..." /></Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Date"><Input type="date" value={date} onChange={(e) => setDate(e.target.value)} /></Field>
          <Field label="Status">
            <select value={status} onChange={(e) => setStatus(e.target.value)} className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm">
              {["PENDING", "CONFIRMED", "CANCELLED", "COMPLETED"].map((s) => (<option key={s} value={s}>{s}</option>))}
            </select>
          </Field>
        </div>
        <Field label="Notes"><Textarea rows={3} value={notes} onChange={(e) => setNotes(e.target.value)} /></Field>
        <div className="flex justify-end gap-2 pt-2"><Button type="button" variant="ghost" onClick={onClose}>Cancel</Button><Button type="submit" loading={busy}>Save</Button></div>
      </form>
    </SidePanel>
  );
}

// ============================================================================
// Theme + Preview
// ============================================================================

export function ThemePage() {
  const { data: templates } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const [theme, setTheme] = useState<Record<string, string>>({ background: "#1a1a1a", foreground: "#fafafa", accent: "#3a6ea4", muted: "#bcbcbc" });
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    if (!tpl) return;
    if (tpl.theme_json) try { const j = JSON.parse(tpl.theme_json); setTheme((t) => ({ ...t, ...j })); } catch {}
  }, [tpl?.id]);
  async function save() {
    if (!tpl) return; setSaving(true);
    try { await call(() => api.patch(`/website/templates/${tpl.id}`, { theme_json: JSON.stringify(theme) })); } catch (e) { setErr(e instanceof Error ? e.message : String(e)); } finally { setSaving(false); }
  }
  return (
    <div className="space-y-6">
      <PageHeader title="Theme" description="Colors and fonts for Template 0." actions={<Button onClick={save} loading={saving} disabled={!tpl}>Save theme</Button>} />
      {err && <Alert>{err}</Alert>}
      <Card className="p-5 space-y-4">
        <div className="grid gap-4 sm:grid-cols-2">
          {(["background", "foreground", "accent", "muted"] as const).map((k) => (
            <Field key={k} label={k}>
              <div className="flex gap-2">
                <input type="color" value={theme[k] ?? "#000000"} onChange={(e) => setTheme((a) => ({ ...a, [k]: e.target.value }))} className="h-9 w-14 rounded border" />
                <Input value={theme[k] ?? ""} onChange={(e) => setTheme((a) => ({ ...a, [k]: e.target.value }))} placeholder="#hex" />
              </div>
            </Field>
          ))}
        </div>
        <Card className="p-6" style={{ background: theme.background, color: theme.foreground, borderColor: theme.accent }}>
          <div className="text-lg font-bold" style={{ fontFamily: "luckiest-guy-regular, cursive" }}>Preview — {theme.accent}</div>
          <div className="mt-1 text-sm" style={{ color: theme.muted }}>KNOWLEDGE + UTOPIA</div>
          <div className="mt-3 inline-block rounded-full px-3 py-1 text-xs text-white" style={{ background: theme.accent }}>Badge</div>
        </Card>
      </Card>
    </div>
  );
}

export function PreviewPage() {
  const { data: templates, err: tplErr } = useList<Template>("/website/templates", "templates");
  const tpl = (templates ?? []).find((t) => t.slug === "template-0") ?? templates?.[0] ?? null;
  const [cfg, setCfg] = useState<Record<string, unknown> | null>(null);
  const [links, setLinks] = useState<WebsiteLink[]>([]);
  const [featuredProducts, setFeaturedProducts] = useState<Product[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!tpl) return;
    let cancelled = false;
    (async () => {
      setLoading(true);
      setErr(null);
      try {
        const r = await api.get<{ sections: WebsiteSection[]; template: Template; links: WebsiteLink[] }>(`/website/templates/${tpl.id}/preview`);
        if (cancelled) return;
        const obj: Record<string, unknown> = {};
        for (const s of r.sections) { try { obj[s.kind] = JSON.parse(s.config_json || "{}"); } catch { obj[s.kind] = {}; } }
        obj["_theme"] = r.template.theme_json ? JSON.parse(r.template.theme_json as string) : {};
        // visibility map
        const vis: Record<string, boolean> = {};
        for (const s of r.sections) vis[s.kind] = s.is_visible;
        obj["_vis"] = vis;
        setCfg(obj);
        setLinks(r.links ?? []);
        // resolve featured products
        const prodIds = (obj["products"] as { featured_product_ids?: string[] } | undefined)?.featured_product_ids ?? [];
        if (prodIds.length) {
          try {
            const pr = await api.get<{ products: Product[] }>("/erp/products");
            const map = new Map(pr.products.map((p) => [p.id, p]));
            setFeaturedProducts(prodIds.map((id) => map.get(id)).filter(Boolean) as Product[]);
          } catch { setFeaturedProducts([]); }
        } else setFeaturedProducts([]);
      } catch (e) { if (!cancelled) setErr(e instanceof Error ? e.message : String(e)); }
      finally { if (!cancelled) setLoading(false); }
    })();
    return () => { cancelled = true; };
  }, [tpl?.id]);

  const visMap = (cfg?.["_vis"] as Record<string, boolean> | undefined) ?? {};
  const isVis = (kind: string) => visMap[kind] !== false;
  const theme = (cfg?.["_theme"] as Record<string, string> | undefined) ?? { background: "#1a1a1a", foreground: "#fafafa", accent: "#3a6ea4", muted: "#bcbcbc" };
  const hero = (cfg?.["hero"] as Record<string, string> | undefined) ?? { title: "K N O W L I A", subtitle: "KNOWLEDGE + UTOPIA", badge: "Version 1.1.1", tagline: "Welcome to Knowlia: where knowledge meets innovation. Experience streamlined course management, intuitive real-time campus navigation, and stress-free organization—all crafted exclusively for COICT students." };
  const dl = (cfg?.["downloads"] as { platforms?: DownloadPlatform[] } | undefined)?.platforms ?? [];
  const slides = (cfg?.["slideshow"] as { slides?: Slide[] } | undefined)?.slides ?? [];

  // ---- loading / empty guards
  if (templates === null) return <div className="space-y-6"><PageHeader title="Preview" description="Live render of Template 0 assembled from all sections (read-only)." /><Spinner /></div>;
  if (tplErr) return <div className="space-y-6"><PageHeader title="Preview" description="Live render of Template 0." /><Alert>{tplErr}</Alert></div>;
  if (!tpl) {
    return (
      <div className="space-y-6">
        <PageHeader title="Preview" description="Live render of Template 0 assembled from all sections (read-only)." />
        <EmptyState message="No website template yet." />
        <Card className="p-6 text-sm text-[var(--app-fg-muted)]">
          Seed Template 0 by running the migration <code className="rounded bg-[var(--app-card-2)] px-1 py-0.5 text-xs">000037_website_core</code> — it inserts <code>template-0</code> with hero/slideshow/downloads/products. Or create one via <code className="rounded bg-[var(--app-card-2)] px-1 py-0.5 text-xs">POST /api/v1/website/templates</code>.
        </Card>
        {/* Demo fallback preview so first-time users see the shape */}
        <div className="overflow-hidden rounded-xl border opacity-60">
          <div className="p-10 text-center" style={{ backgroundColor: "#1a1a1a", color: "#fafafa", backgroundImage: "linear-gradient(to right,#fafafa18 1px,transparent 1px),linear-gradient(to bottom,#fafafa18 1px,transparent 1px)", backgroundSize: "20px 20px" }}>
            <h1 className="text-5xl font-bold" style={{ fontFamily: "luckiest-guy-regular, cursive" }}>K N O W L I A</h1>
            <p className="mt-2 text-2xl opacity-60" style={{ fontFamily: "sister-spray, cursive" }}>KNOWLEDGE + UTOPIA</p>
            <span className="mt-3 inline-block rounded-full bg-[#3a6ea4] px-3 py-1 text-xs text-white">Version 1.1.1</span>
          </div>
          <div className="p-6"><EmptyState message="Create Template 0 to see live data here." /></div>
        </div>
      </div>
    );
  }
  if (loading && !cfg) return <div className="space-y-6"><PageHeader title="Preview" description="Live render of Template 0." /><Spinner /></div>;

  return (
    <div className="space-y-6">
      <PageHeader title="Preview" description={`Live render of ${tpl.name} · ${tpl.slug} · v${tpl.version} (read-only). Toggle visibility in each sub-module.`} actions={<Badge tone={tpl.is_active ? "green" : "zinc"}>{tpl.is_active ? "active" : "inactive"}</Badge>} />
      {err && <Alert tone="info">{err}</Alert>}
      <div className="overflow-hidden rounded-xl border border-[var(--app-border)] shadow-sm">
        {/* Top nav bar from links */}
        {isVis("links") !== false && (
          <nav className="flex flex-wrap items-center gap-1 border-b border-[var(--app-border)] bg-[var(--app-card)] px-4 py-2">
            {links.length === 0 ? (
              <span className="text-xs text-[var(--app-fg-muted)]">No links — add some in <NavLink to="/website/links" className="underline text-[var(--accent)]">Website → Links</NavLink></span>
            ) : links.filter((l) => l.is_visible).map((l) => (
              <a key={l.id} href={l.href} className="rounded-full px-3 py-1 text-xs font-medium hover:bg-[var(--app-card-2)] border border-transparent hover:border-[var(--app-border)]">{l.label}</a>
            ))}
            {links.length > 0 && links.filter((l) => !l.is_visible).length > 0 && <span className="ml-2 text-[10px] text-[var(--app-fg-muted)]">+{links.filter((l) => !l.is_visible).length} hidden</span>}
          </nav>
        )}

        {/* Hero */}
        {isVis("hero") ? (
          <div className="relative overflow-hidden p-10 text-center" style={{ backgroundColor: theme.background, color: theme.foreground, backgroundImage: "linear-gradient(to right,#fafafa18 1px,transparent 1px),linear-gradient(to bottom,#fafafa18 1px,transparent 1px)", backgroundSize: "20px 20px" }}>
            <h1 className="text-5xl font-bold tracking-tight" style={{ fontFamily: "luckiest-guy-regular, cursive" }}>{hero.title ?? "K N O W L I A"}</h1>
            <p className="mt-2 text-2xl opacity-60" style={{ fontFamily: "sister-spray, cursive", color: theme.muted }}>{hero.subtitle ?? "KNOWLEDGE + UTOPIA"}</p>
            <span className="mt-3 inline-block rounded-full px-3 py-1 text-xs text-white" style={{ background: theme.accent }}>{hero.badge ?? "Version 1.1.1"}</span>
            <p className="mx-auto mt-6 max-w-2xl text-sm leading-relaxed opacity-80">{hero.tagline ?? "Welcome to Knowlia..."}</p>
          </div>
        ) : (
          <div className="border-y border-dashed bg-[var(--app-card-2)] p-6 text-center text-xs text-[var(--app-fg-muted)]">Hero hidden — enable in <NavLink to="/website/hero" className="underline">Website → Hero</NavLink></div>
        )}

        {/* Slideshow */}
        {isVis("slideshow") !== false && (
          <section className="border-t border-[var(--app-border)] bg-black p-4">
            <div className="mx-auto max-w-5xl">
              <div className="mb-2 flex items-center justify-between">
                <h3 className="text-xs font-semibold uppercase tracking-wider text-white/70">Slideshow</h3>
                <span className="text-[10px] text-white/40">{slides.length} slides</span>
              </div>
              {slides.length === 0 ? (
                <div className="rounded-lg border border-dashed border-white/20 p-8 text-center text-sm text-white/50">
                  No slides yet — add images in <NavLink to="/website/slideshow" className="underline text-white">Website → Slideshow</NavLink>
                </div>
              ) : (
                <div className="grid gap-3 sm:grid-cols-2">
                  {slides.slice(0, 4).map((s, i) => (
                    <div key={i} className="overflow-hidden rounded-lg border border-white/10 bg-zinc-900">
                      {s.url ? <img src={s.url} alt={s.alt ?? `Slide ${i + 1}`} className="aspect-[16/9] w-full object-cover" loading="lazy" onError={(e) => { (e.currentTarget as HTMLImageElement).style.display = "none"; }} /> : <div className="grid aspect-[16/9] place-items-center text-xs text-white/40">No image</div>}
                      <div className="px-2 py-1 text-[10px] text-white/50 truncate">{s.alt ?? s.url}</div>
                    </div>
                  ))}
                  {slides.length > 4 && <div className="col-span-full text-center text-xs text-white/40">+{slides.length - 4} more</div>}
                </div>
              )}
            </div>
          </section>
        )}

        {/* Downloads */}
        {isVis("downloads") !== false && (
          <section className="border-t border-[var(--app-border)] bg-[#fafafa] p-6 dark:bg-[#1a1a1a]">
            <div className="mx-auto max-w-5xl">
              <h3 className="mb-3 text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">Downloads</h3>
              {dl.length === 0 ? (
                <div className="rounded-lg border border-dashed p-8 text-center"><EmptyState message="No download platforms configured." /></div>
              ) : (
                <div className="grid gap-4 md:grid-cols-2">
                  {dl.map((p) => (
                    <Card key={p.platform} className="p-4">
                      <div className="text-sm font-semibold">{p.platform || "Untitled"}</div>
                      <div className="mt-2 space-y-1">
                        {p.versions.length === 0 ? <div className="text-xs text-[var(--app-fg-muted)]">No versions</div> : p.versions.map((v) => (
                          <a key={v.arch + v.url} href={v.url || "#"} target="_blank" rel="noreferrer" className="flex items-center justify-between rounded border px-2 py-1.5 text-xs hover:bg-[var(--app-card-2)]">
                            <span className="flex items-center gap-1.5"><span>{v.arch || "—"}</span>{v.recommended && <Badge tone="green">recommended</Badge>}</span>
                            <span className="text-[var(--accent)]">↓ download</span>
                          </a>
                        ))}
                      </div>
                    </Card>
                  ))}
                </div>
              )}
            </div>
          </section>
        )}

        {/* Products showcase */}
        {isVis("products") !== false && (
          <section className="border-t border-[var(--app-border)] bg-white p-6 dark:bg-zinc-900">
            <div className="mx-auto max-w-5xl">
              <div className="flex items-center justify-between">
                <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">Products showcase</h3>
                <NavLink to="/website/products" className="text-xs text-[var(--accent)] underline">manage →</NavLink>
              </div>
              {featuredProducts.length === 0 ? (
                <div className="mt-3 rounded-lg border border-dashed p-8 text-center text-sm text-[var(--app-fg-muted)]">
                  No featured products — pick some in <NavLink to="/website/products" className="underline text-[var(--accent)]">Website → Products</NavLink>
                </div>
              ) : (
                <div className="mt-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                  {featuredProducts.map((p) => (
                    <Card key={p.id} className="p-4">
                      <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 text-[10px]">{p.sku}</code>
                      <div className="mt-1 text-sm font-semibold leading-tight">{p.name}</div>
                      <div className="mt-1 line-clamp-2 text-xs text-[var(--app-fg-muted)]">{p.description ?? "—"}</div>
                      <Badge tone={p.is_active ? "green" : "zinc"}>{p.is_active ? "active" : "inactive"}</Badge>
                    </Card>
                  ))}
                </div>
              )}
            </div>
          </section>
        )}

        {/* Footer */}
        <div className="border-t border-[var(--app-border)] bg-[var(--app-card-2)] p-4 text-center text-xs text-[var(--app-fg-muted)]">
          {featuredProducts.length ? `Featured: ${featuredProducts.length} products · ` : ""}
          Template 0 · {tpl.version} · v{theme.accent} accent · <NavLink to="/website/theme" className="underline">Theme</NavLink>
        </div>
      </div>
      <Card className="p-4 text-xs text-[var(--app-fg-muted)]">
        Public snapshot endpoint: <code className="rounded bg-[var(--app-card-2)] px-1 py-0.5">GET /api/v1/website/templates/{tpl.id}/preview</code> — currently requires <code>website:read</code>; move the route to <code>public()</code> in <code>routes/src/lib.rs</code> to make it storefront-public.
      </Card>
    </div>
  );
}
