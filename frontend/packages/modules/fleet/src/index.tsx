// Fleet module — Bolt login, MarineTraffic cookie viewer, and the in-memory
// poller snapshot. The Fleet page does NOT keep a stored list of vehicles —
// the poller in `gateway-fleet` holds the live snapshot in process memory
// only, and the Map page can render that snapshot as MVT tiles via
// `/api/v1/fleet/tiles/{source}/{z}/{x}/{y}`.
import { useCallback, useEffect, useState } from "react";
import { api } from "@gateway/lib";
import {
  Alert,
  Badge,
  Button,
  Card,
  Field,
  Input,
  PageHeader,
  SidePanel,
  Spinner,
  Textarea,
} from "@gateway/ui";

type LiveSource = { source: string; fetched_at: string; count: number };
type LiveSummary = {
  success: boolean;
  cached: boolean;
  counts: { bolt: number; marine: number; flights: number; total: number };
  sources: LiveSource[];
  cells: unknown;
  updated_at?: string;
};

type BoltStatus = {
  success: boolean;
  logged_in: boolean;
  meta?: { phone?: string; logged_in_at?: string } | null;
};
type MarineCookie = {
  success: boolean;
  configured: boolean;
  cookie: string | null;
  live_marine_count: number;
};

export default function FleetPage() {
  const [summary, setSummary] = useState<LiveSummary | null>(null);
  const [bolt, setBolt] = useState<BoltStatus | null>(null);
  const [marineCookie, setMarineCookie] = useState<MarineCookie | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);

  const [showBoltStart, setShowBoltStart] = useState(false);
  const [showBoltConfirm, setShowBoltConfirm] = useState(false);
  const [showCookie, setShowCookie] = useState(false);

  const load = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const [s, b, m] = await Promise.all([
        api.get<LiveSummary>("/fleet/summary"),
        api.get<BoltStatus>("/fleet/bolt/status"),
        api.get<MarineCookie>("/fleet/marine/cookie"),
      ]);
      setSummary(s);
      setBolt(b);
      setMarineCookie(m);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load");
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void load();
    const t = setInterval(() => void load(), 30_000);
    return () => clearInterval(t);
  }, [load]);

  return (
    <div className="space-y-6">
      <PageHeader
        title="Fleet"
        description="Bolt login + MarineTraffic cookie. Live vehicle data is held in memory by the gateway poller — no fleet tables are written."
        actions={<Button onClick={() => void load()}>Refresh</Button>}
      />
      {error && <Alert tone="error">{error}</Alert>}

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            Live snapshot
          </h3>
          {busy && !summary ? (
            <div className="mt-3">
              <Spinner />
            </div>
          ) : summary ? (
            <div className="mt-3 space-y-2 text-sm">
              <Row k="Cached" v={summary.cached ? "yes" : "no"} />
              <Row k="Bolt" v={summary.counts.bolt} />
              <Row k="Marine" v={summary.counts.marine} />
              <Row k="Flights" v={summary.counts.flights} />
              <Row k="Total" v={summary.counts.total} />
              {summary.updated_at && <Row k="Updated" v={new Date(summary.updated_at).toLocaleString()} />}
              {summary.sources.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {summary.sources.map((s) => (
                    <Badge key={s.source}>
                      {s.source}: {s.count}
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          ) : (
            <Empty msg="No snapshot yet." />
          )}
        </Card>

        <Card className="p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            Bolt
          </h3>
          {bolt ? (
            <div className="mt-3 space-y-2 text-sm">
              <Row k="Status" v={bolt.logged_in ? "logged in" : "not logged in"} />
              {bolt.meta?.phone && <Row k="Phone" v={bolt.meta.phone} />}
              {bolt.meta?.logged_in_at && (
                <Row k="Since" v={new Date(bolt.meta.logged_in_at).toLocaleString()} />
              )}
            </div>
          ) : (
            <div className="mt-3">
              <Spinner />
            </div>
          )}
          <div className="mt-4 flex flex-wrap gap-2">
            <Button onClick={() => setShowBoltStart(true)}>Start login</Button>
          </div>
        </Card>

        <Card className="p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">
            MarineTraffic cookie
          </h3>
          {marineCookie ? (
            <div className="mt-3 space-y-2 text-sm">
              <Row k="Configured" v={marineCookie.configured ? "yes" : "no"} />
              <Row k="Live marine" v={marineCookie.live_marine_count} />
            </div>
          ) : (
            <div className="mt-3">
              <Spinner />
            </div>
          )}
          <div className="mt-4 flex flex-wrap gap-2">
            <Button onClick={() => setShowCookie(true)}>View / set cookie</Button>
          </div>
        </Card>
      </div>

      <Alert tone="info">
        <strong>No vehicle list is kept.</strong> The poller holds the live snapshot in memory and refreshes it every few seconds. The Map
        page can render that snapshot as MVT tiles via{" "}
        <code>/api/v1/fleet/tiles/&lt;source&gt;/&lt;z&gt;/&lt;x&gt;/&lt;y&gt;</code> (sources: <code>bolt</code>, <code>marine</code>, <code>flights</code>).
      </Alert>

      <SidePanel
        open={showBoltStart}
        onClose={() => setShowBoltStart(false)}
        title="Bolt login — start"
      >
        <BoltStartForm
          onDone={() => {
            setShowBoltStart(false);
            setShowBoltConfirm(true);
            void load();
          }}
          onCancel={() => setShowBoltStart(false)}
        />
      </SidePanel>
      <SidePanel
        open={showBoltConfirm}
        onClose={() => setShowBoltConfirm(false)}
        title="Bolt login — enter code"
      >
        <BoltConfirmForm
          onDone={() => {
            setShowBoltConfirm(false);
            void load();
          }}
          onCancel={() => setShowBoltConfirm(false)}
        />
      </SidePanel>
      <SidePanel open={showCookie} onClose={() => setShowCookie(false)} title="MarineTraffic cookie">
        <CookieForm
          existing={marineCookie?.cookie ?? null}
          onDone={() => {
            setShowCookie(false);
            void load();
          }}
          onCancel={() => setShowCookie(false)}
        />
      </SidePanel>
    </div>
  );
}

function Row({ k, v }: { k: string; v: string | number | boolean }) {
  return (
    <div className="flex items-center justify-between text-sm">
      <span className="text-[var(--app-fg-muted)]">{k}</span>
      <span className="font-medium">{String(v)}</span>
    </div>
  );
}

function Empty({ msg }: { msg: string }) {
  return (
    <div className="mt-3 rounded border border-dashed border-[var(--app-border)] px-3 py-4 text-center text-xs text-[var(--app-fg-muted)]">
      {msg}
    </div>
  );
}

function BoltStartForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [phone, setPhone] = useState("");
  const [channel, setChannel] = useState<"sms" | "email">("sms");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setErr(null);
    try {
      await api.post("/fleet/bolt/login/start", { phone, channel });
      onDone();
    } catch (e) {
      setErr(e instanceof Error ? e.message : "failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit} className="space-y-4">
      {err && <Alert tone="error">{err}</Alert>}
      <Field label="Phone (E.164)">
        <Input
          required
          placeholder="+255712345678"
          value={phone}
          onChange={(e) => setPhone(e.target.value)}
        />
      </Field>
      <Field label="Channel">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
          value={channel}
          onChange={(e) => setChannel(e.target.value as "sms" | "email")}
        >
          <option value="sms">SMS</option>
          <option value="email">Email</option>
        </select>
      </Field>
      <div className="flex justify-end gap-2 pt-2">
        <Button type="button" variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" loading={busy}>
          Send code
        </Button>
      </div>
    </form>
  );
}

function BoltConfirmForm({ onDone, onCancel }: { onDone: () => void; onCancel: () => void }) {
  const [otp, setOtp] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setErr(null);
    try {
      await api.post("/fleet/bolt/login/confirm", { otp });
      onDone();
    } catch (e) {
      setErr(e instanceof Error ? e.message : "failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit} className="space-y-4">
      {err && <Alert tone="error">{err}</Alert>}
      <Field label="OTP code">
        <Input required value={otp} onChange={(e) => setOtp(e.target.value)} />
      </Field>
      <div className="flex justify-end gap-2 pt-2">
        <Button type="button" variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" loading={busy}>
          Confirm
        </Button>
      </div>
    </form>
  );
}

function CookieForm({
  existing,
  onDone,
  onCancel,
}: {
  existing: string | null;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [cookie, setCookie] = useState(existing ?? "");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [showFull, setShowFull] = useState(false);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    setBusy(true);
    setErr(null);
    try {
      await api.post("/fleet/marine/cookie", { cookie });
      onDone();
    } catch (e) {
      setErr(e instanceof Error ? e.message : "failed");
    } finally {
      setBusy(false);
    }
  }

  async function clear() {
    setBusy(true);
    setErr(null);
    try {
      await api.post("/fleet/marine/cookie", { cookie: "" });
      setCookie("");
      onDone();
    } catch (e) {
      setErr(e instanceof Error ? e.message : "failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form onSubmit={submit} className="space-y-4">
      {err && <Alert tone="error">{err}</Alert>}
      <Field label="Cookie value">
        <Textarea
          rows={showFull ? 10 : 4}
          value={cookie}
          onChange={(e) => setCookie(e.target.value)}
          placeholder="Paste the Cloudflare cf_bm cookie here"
          className="font-mono text-xs"
        />
      </Field>
      <label className="flex items-center gap-2 text-xs text-[var(--app-fg-muted)]">
        <input type="checkbox" checked={showFull} onChange={(e) => setShowFull(e.target.checked)} />
        Show full value
      </label>
      <div className="flex justify-between gap-2 pt-2">
        <Button type="button" variant="danger" onClick={() => void clear()}>
          Clear
        </Button>
        <div className="flex gap-2">
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" loading={busy}>
            Save
          </Button>
        </div>
      </div>
    </form>
  );
}
