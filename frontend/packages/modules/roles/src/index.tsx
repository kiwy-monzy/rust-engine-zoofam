import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import type { Permission } from "@gateway/lib";
import {
  Alert,
  Badge,
  Button,
  EmptyState,
  Field,
  Input,
  PageHeader,
  PanelSection,
  SidePanel,
  Spinner,
  Table,
  Td,
  cn,
} from "@gateway/ui";

interface RoleWithPermissions {
  id: number;
  name: string;
  description: string;
  permissions: Permission[];
}

export function RolesPage({
  canWriteRoles = false,
  canReadPermissions = false,
}: {
  canWriteRoles?: boolean;
  canReadPermissions?: boolean;
}) {
  const [roles, setRoles] = useState<RoleWithPermissions[] | null>(null);
  const [perms, setPerms] = useState<Permission[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [panelError, setPanelError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [pickPerm, setPickPerm] = useState("");
  const [nameDraft, setNameDraft] = useState("");
  const [descDraft, setDescDraft] = useState("");

  const selected = roles?.find((r) => r.id === selectedId) ?? null;
  const heldIds = new Set(selected?.permissions.map((p) => p.id) ?? []);
  const grantable = (perms ?? []).filter((p) => !heldIds.has(p.id));

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const res = await api.get<{ roles: RoleWithPermissions[] }>("/roles");
      setRoles(res.roles);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load roles");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function loadPerms() {
    if (!canReadPermissions || perms) return;
    try {
      const res = await api.get<{ permissions: Permission[] }>("/permissions");
      setPerms(res.permissions);
    } catch {
      setPerms([]);
    }
  }

  function openPanel(r: RoleWithPermissions) {
    setSelectedId(r.id);
    setPanelError(null);
    setPickPerm("");
    setNameDraft(r.name);
    setDescDraft(r.description);
    void loadPerms();
  }

  function closePanel() {
    setSelectedId(null);
    setPanelError(null);
  }

  async function run(fn: () => Promise<unknown>): Promise<boolean> {
    setSaving(true);
    setPanelError(null);
    try {
      await fn();
      await load();
      return true;
    } catch (e) {
      setPanelError(e instanceof Error ? e.message : "the change failed");
      return false;
    } finally {
      setSaving(false);
    }
  }

  function grant() {
    if (!selected || !pickPerm) return;
    void run(() =>
      api.post(`/roles/${selected.id}/permissions/${Number(pickPerm)}`),
    ).then((ok) => ok && setPickPerm(""));
  }

  function revoke(p: Permission) {
    if (!selected) return;
    void run(() => api.del(`/roles/${selected.id}/permissions/${p.id}`));
  }

  function removeRole() {
    if (!selected) return;
    if (!confirm(`Delete role "${selected.name}"? Users holding it lose its permissions.`))
      return;
    void run(() => api.del(`/roles/${selected.id}`)).then((ok) => ok && closePanel());
  }

  function removeRow(r: RoleWithPermissions) {
    if (!confirm(`Delete role "${r.name}"? Users holding it lose its permissions.`)) return;
    void run(() => api.del(`/roles/${r.id}`)).then((ok) => {
      if (ok && selectedId === r.id) closePanel();
    });
  }

  const identityDirty =
    !!selected &&
    (nameDraft.trim() !== selected.name || descDraft.trim() !== selected.description);

  function saveIdentity() {
    if (!selected) return;
    void run(() =>
      api.patch(`/roles/${selected.id}`, {
        name: nameDraft.trim(),
        description: descDraft.trim(),
      }),
    );
  }

  function resetIdentity() {
    if (!selected) return;
    setNameDraft(selected.name);
    setDescDraft(selected.description);
    setPanelError(null);
  }

  return (
    <div className="space-y-5">
      <PageHeader
        title="Roles"
        description="Bundles of permissions assigned to users. Select a role to manage its permissions."
        actions={
          <Button variant="ghost" onClick={() => void load()} loading={busy}>
            Refresh
          </Button>
        }
      />
      {error && <Alert>{error}</Alert>}
      {!roles && !error && (
        <div className="grid place-items-center py-16">
          <Spinner />
        </div>
      )}
      {roles && roles.length === 0 && <EmptyState message="No roles defined." />}
      {roles && roles.length > 0 && (
        <Table head={["Role", "Description", "Permissions", ""]}>
          {roles.map((r) => (
            <tr
              key={r.id}
              onClick={() => openPanel(r)}
              className={cn(
                "cursor-pointer transition-colors",
                r.id === selectedId
                  ? "bg-[var(--accent-soft)]"
                  : "hover:bg-[var(--sb-hover)]",
              )}
            >
              <Td>
                <span className="flex items-center gap-2 font-medium text-[var(--app-fg)]">
                  {r.name}
                  <Badge tone={r.name === "admin" ? "red" : "zinc"}>#{r.id}</Badge>
                </span>
              </Td>
              <Td className="text-[var(--app-fg-muted)]">{r.description}</Td>
              <Td>
                <span className="flex flex-wrap gap-1">
                  {r.permissions.length === 0 ? (
                    <span className="text-[var(--app-fg-faint)]">none bound</span>
                  ) : (
                    r.permissions.slice(0, 4).map((p) => (
                      <code
                        key={p.id}
                        className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 font-mono text-xs text-[var(--accent-fg)]"
                      >
                        {p.module}:{p.action}
                      </code>
                    ))
                  )}
                  {r.permissions.length > 4 && (
                    <span className="text-xs text-[var(--app-fg-faint)]">
                      +{r.permissions.length - 4} more
                    </span>
                  )}
                </span>
              </Td>
              <Td className="text-right">
                <span className="flex items-center justify-end gap-2">
                  <span className="text-xs text-[var(--app-fg-faint)]">manage ›</span>
                  {canWriteRoles && (
                    <Button
                      variant="danger"
                      className="px-2.5 py-1 text-xs"
                      disabled={saving}
                      onClick={(e) => {
                        e.stopPropagation();
                        removeRow(r);
                      }}
                    >
                      Delete
                    </Button>
                  )}
                </span>
              </Td>
            </tr>
          ))}
        </Table>
      )}

      <SidePanel
        open={!!selected}
        onClose={closePanel}
        title={selected ? `Role: ${selected.name}` : ""}
        subtitle={selected?.description}
      >
        {selected && (
          <>
            {panelError && <Alert>{panelError}</Alert>}

            <PanelSection label="Role details">
              {canWriteRoles ? (
                <>
                  <Field label="Name">
                    <Input
                      value={nameDraft}
                      disabled={saving}
                      onChange={(e) => setNameDraft(e.target.value)}
                    />
                  </Field>
                  <Field label="Description">
                    <Input
                      value={descDraft}
                      disabled={saving}
                      onChange={(e) => setDescDraft(e.target.value)}
                    />
                  </Field>
                  <div className="flex gap-2">
                    <Button
                      className="flex-1"
                      disabled={!identityDirty}
                      loading={saving}
                      onClick={saveIdentity}
                    >
                      Save
                    </Button>
                    <Button
                      variant="ghost"
                      className="flex-1"
                      disabled={!identityDirty || saving}
                      onClick={resetIdentity}
                    >
                      Cancel
                    </Button>
                  </div>
                </>
              ) : (
                <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
                  <dt className="text-[var(--app-fg-faint)]">Name</dt>
                  <dd className="font-medium text-[var(--app-fg)]">{selected.name}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Role id</dt>
                  <dd className="font-mono text-[var(--app-fg-muted)]">#{selected.id}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Permissions</dt>
                  <dd className="text-[var(--app-fg-muted)]">{selected.permissions.length} bound</dd>
                </dl>
              )}
            </PanelSection>

            <PanelSection label="Permissions">
              <div className="space-y-1.5">
                {selected.permissions.length === 0 && (
                  <p className="text-sm text-[var(--app-fg-faint)]">No permissions bound.</p>
                )}
                {selected.permissions.map((p) => (
                  <div
                    key={p.id}
                    className="flex items-center justify-between gap-2 rounded-xl border border-[var(--app-border)] px-3 py-2"
                  >
                    <span className="min-w-0">
                      <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 font-mono text-xs text-[var(--accent-fg)]">
                        {p.module}:{p.action}
                      </code>
                      <span className="mt-1 block truncate text-xs text-[var(--app-fg-muted)]">
                        {p.description}
                      </span>
                    </span>
                    {canWriteRoles && (
                      <button
                        type="button"
                        title={`revoke ${p.module}:${p.action}`}
                        disabled={saving}
                        onClick={() => revoke(p)}
                        className="grid h-6 w-6 shrink-0 place-items-center rounded-md text-[var(--app-fg-faint)] transition-colors hover:bg-red-500/15 hover:text-red-500"
                      >
                        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round">
                          <line x1="6" y1="6" x2="18" y2="18" />
                          <line x1="18" y1="6" x2="6" y2="18" />
                        </svg>
                      </button>
                    )}
                  </div>
                ))}
              </div>

              {canWriteRoles && grantable.length > 0 && (
                <div className="flex items-center gap-2 pt-1">
                  <select
                    aria-label="permission to grant"
                    value={pickPerm}
                    disabled={saving}
                    onChange={(e) => setPickPerm(e.target.value)}
                    className="min-w-0 flex-1 rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-2 py-1.5 text-sm text-[var(--app-fg)] focus:border-[var(--accent)] focus:outline-none disabled:opacity-50"
                  >
                    <option value="" disabled>
                      grant a permission…
                    </option>
                    {grantable.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.module}:{p.action}
                      </option>
                    ))}
                  </select>
                  <Button
                    disabled={!pickPerm}
                    loading={saving}
                    onClick={grant}
                    className="px-3 py-1.5"
                  >
                    Grant
                  </Button>
                </div>
              )}
            </PanelSection>

            {canWriteRoles && (
              <PanelSection label="Danger zone">
                <Button variant="danger" className="w-full" loading={saving} onClick={removeRole}>
                  Delete role
                </Button>
              </PanelSection>
            )}
          </>
        )}
      </SidePanel>
    </div>
  );
}
