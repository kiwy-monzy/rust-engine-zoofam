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

interface RoleSummary {
  id: number;
  name: string;
  permissions: Permission[];
}

export function PermissionsPage({
  canWritePermissions = false,
  canReadRoles = false,
}: {
  canWritePermissions?: boolean;
  canReadRoles?: boolean;
}) {
  const [perms, setPerms] = useState<Permission[] | null>(null);
  const [roles, setRoles] = useState<RoleSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [panelError, setPanelError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [moduleDraft, setModuleDraft] = useState("");
  const [actionDraft, setActionDraft] = useState("");
  const [descDraft, setDescDraft] = useState("");

  const selected = perms?.find((p) => p.id === selectedId) ?? null;
  const heldBy = (roles ?? []).filter((r) =>
    r.permissions.some((p) => p.id === selectedId),
  );

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const res = await api.get<{ permissions: Permission[] }>("/permissions");
      setPerms(res.permissions);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load permissions");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function loadRoles() {
    if (!canReadRoles || roles) return;
    try {
      const res = await api.get<{ roles: RoleSummary[] }>("/roles");
      setRoles(res.roles);
    } catch {
      setRoles([]);
    }
  }

  function openPanel(p: Permission) {
    setSelectedId(p.id);
    setPanelError(null);
    setModuleDraft(p.module);
    setActionDraft(p.action);
    setDescDraft(p.description);
    void loadRoles();
  }

  function closePanel() {
    setSelectedId(null);
    setPanelError(null);
  }

  function remove() {
    if (!selected) return;
    if (!confirm(`Delete permission ${selected.module}:${selected.action}? Roles lose it immediately.`))
      return;
    setSaving(true);
    setPanelError(null);
    api
      .del(`/permissions/${selected.id}`)
      .then(async () => {
        await load();
        closePanel();
      })
      .catch((e) =>
        setPanelError(e instanceof Error ? e.message : "delete failed"),
      )
      .finally(() => setSaving(false));
  }

  function removeRow(p: Permission) {
    if (!confirm(`Delete permission ${p.module}:${p.action}? Roles lose it immediately.`)) return;
    setSaving(true);
    setError(null);
    api
      .del(`/permissions/${p.id}`)
      .then(() => load())
      .catch((e) => setError(e instanceof Error ? e.message : "delete failed"))
      .finally(() => setSaving(false));
  }

  const identityDirty =
    !!selected &&
    (moduleDraft.trim() !== selected.module ||
      actionDraft.trim() !== selected.action ||
      descDraft.trim() !== selected.description);

  function saveIdentity() {
    if (!selected) return;
    setSaving(true);
    setPanelError(null);
    api
      .patch(`/permissions/${selected.id}`, {
        module: moduleDraft.trim(),
        action: actionDraft.trim(),
        description: descDraft.trim(),
      })
      .then(() => load())
      .catch((e) =>
        setPanelError(e instanceof Error ? e.message : "saving failed"),
      )
      .finally(() => setSaving(false));
  }

  function resetIdentity() {
    if (!selected) return;
    setModuleDraft(selected.module);
    setActionDraft(selected.action);
    setDescDraft(selected.description);
    setPanelError(null);
  }

  return (
    <div className="space-y-5">
      <PageHeader
        title="Permissions"
        description="module:action pairs the JWT is stamped with at login. Select one for details."
        actions={
          <Button variant="ghost" onClick={() => void load()} loading={busy}>
            Refresh
          </Button>
        }
      />
      {error && <Alert>{error}</Alert>}
      {!perms && !error && (
        <div className="grid place-items-center py-16">
          <Spinner />
        </div>
      )}
      {perms && perms.length === 0 && <EmptyState message="No permissions defined." />}
      {perms && perms.length > 0 && (
        <Table head={["Module", "Action", "Key", "Description", ""]}>
          {perms.map((p) => (
            <tr
              key={p.id}
              onClick={() => openPanel(p)}
              className={cn(
                "cursor-pointer transition-colors",
                p.id === selectedId
                  ? "bg-[var(--accent-soft)]"
                  : "hover:bg-[var(--sb-hover)]",
              )}
            >
              <Td><Badge tone="red">{p.module}</Badge></Td>
              <Td><Badge>{p.action}</Badge></Td>
              <Td>
                <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 font-mono text-xs text-[var(--accent-fg)]">{p.module}:{p.action}</code>
              </Td>
              <Td className="text-[var(--app-fg-muted)]">{p.description}</Td>
              <Td className="text-right">
                {canWritePermissions && (
                  <Button
                    variant="danger"
                    className="px-2.5 py-1 text-xs"
                    disabled={saving}
                    onClick={(e) => {
                      e.stopPropagation();
                      removeRow(p);
                    }}
                  >
                    Delete
                  </Button>
                )}
              </Td>
            </tr>
          ))}
        </Table>
      )}

      <SidePanel
        open={!!selected}
        onClose={closePanel}
        title={selected ? `${selected.module}:${selected.action}` : ""}
        subtitle="Permission"
      >
        {selected && (
          <>
            {panelError && <Alert>{panelError}</Alert>}

            <PanelSection label="Permission details">
              {canWritePermissions ? (
                <>
                  <Field label="Module">
                    <Input
                      value={moduleDraft}
                      disabled={saving}
                      onChange={(e) => setModuleDraft(e.target.value)}
                    />
                  </Field>
                  <Field label="Action">
                    <Input
                      value={actionDraft}
                      disabled={saving}
                      onChange={(e) => setActionDraft(e.target.value)}
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
                  <dt className="text-[var(--app-fg-faint)]">Key</dt>
                  <dd>
                    <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 font-mono text-xs text-[var(--accent-fg)]">
                      {selected.module}:{selected.action}
                    </code>
                  </dd>
                  <dt className="text-[var(--app-fg-faint)]">Permission id</dt>
                  <dd className="font-mono text-[var(--app-fg-muted)]">#{selected.id}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Description</dt>
                  <dd className="text-[var(--app-fg-muted)]">{selected.description || "—"}</dd>
                </dl>
              )}
            </PanelSection>

            {canReadRoles && (
              <PanelSection label="Held by roles">
                {heldBy.length === 0 ? (
                  <p className="text-sm text-[var(--app-fg-faint)]">
                    Not bound to any role.
                  </p>
                ) : (
                  <div className="flex flex-wrap gap-1.5">
                    {heldBy.map((r) => (
                      <Badge key={r.id} tone={r.name === "admin" ? "red" : "zinc"}>
                        {r.name}
                      </Badge>
                    ))}
                  </div>
                )}
                <p className="text-xs leading-relaxed text-[var(--app-fg-muted)]">
                  Grant or revoke this from the Roles page — permissions attach to
                  roles, and roles attach to users.
                </p>
              </PanelSection>
            )}

            {canWritePermissions && (
              <PanelSection label="Danger zone">
                <Button variant="danger" className="w-full" loading={saving} onClick={remove}>
                  Delete permission
                </Button>
              </PanelSection>
            )}
          </>
        )}
      </SidePanel>
    </div>
  );
}
