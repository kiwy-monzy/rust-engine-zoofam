import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import type { SessionUser } from "@gateway/lib";
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

interface UserWithRoles extends SessionUser {
  created_at: string;
  roles: string[];
}

interface RoleOption {
  id: number;
  name: string;
  description: string;
  permissions: { id: number; module: string; action: string; description: string }[];
}

type SaveKind = "role" | "status" | "delete";

export function UsersPage({
  canWriteUsers = false,
  canReadRoles = false,
}: {
  canWriteUsers?: boolean;
  canReadRoles?: boolean;
}) {
  const [users, setUsers] = useState<UserWithRoles[] | null>(null);
  const [roles, setRoles] = useState<RoleOption[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [roleChoice, setRoleChoice] = useState<string>("");
  const [panelError, setPanelError] = useState<string | null>(null);
  const [saving, setSaving] = useState<SaveKind | null>(null);
  const [nameDraft, setNameDraft] = useState("");
  const [firstNameDraft, setFirstNameDraft] = useState("");
  const [middleNameDraft, setMiddleNameDraft] = useState("");
  const [lastNameDraft, setLastNameDraft] = useState("");
  const [usernameDraft, setUsernameDraft] = useState("");
  const [avatarDraft, setAvatarDraft] = useState("");

  const selected = users?.find((u) => u.id === selectedId) ?? null;
  const currentRole = selected?.roles[0] ?? "";
  const roleUnchanged = (selected?.roles[0] ?? "") === roleChoice;

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const res = await api.get<{ users: UserWithRoles[] }>("/users");
      setUsers(res.users);
      if (canReadRoles) {
        const r = await api.get<{
          roles: { role: { id: number; name: string; description: string }; permissions: RoleOption["permissions"] }[];
        }>("/roles");
        setRoles(r.roles.map((item) => ({ ...item.role, permissions: item.permissions })));
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load users");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  function openPanel(u: UserWithRoles) {
    setSelectedId(u.id);
    setRoleChoice(u.roles[0] ?? "");
    setPanelError(null);
    setNameDraft(u.display_name);
    setFirstNameDraft(u.first_name ?? "");
    setMiddleNameDraft(u.middle_name ?? "");
    setLastNameDraft(u.last_name ?? "");
    setUsernameDraft(u.username ?? "");
    setAvatarDraft(u.avatar_url);
  }

  function closePanel() {
    setSelectedId(null);
    setPanelError(null);
  }

  async function run(kind: SaveKind, fn: () => Promise<unknown>) {
    setSaving(kind);
    setPanelError(null);
    try {
      await fn();
      await load();
      return true;
    } catch (e) {
      setPanelError(e instanceof Error ? e.message : "the change failed");
      return false;
    } finally {
      setSaving(null);
    }
  }

  function saveRole() {
    if (!selected) return;
    if (roleChoice === "") {
      const roleId = roles?.find((r) => r.name === currentRole)?.id;
      if (!roleId) return;
      void run("role", () => api.del(`/users/${selected.id}/roles/${roleId}`));
    } else {
      void run("role", () =>
        api.post(`/users/${selected.id}/roles/${Number(roleChoice)}`),
      );
    }
  }

  function toggleActive() {
    if (!selected) return;
    void run("status", () =>
      api.patch(`/users/${selected.id}`, { is_active: !selected.is_active }),
    );
  }

  function removeUser() {
    if (!selected) return;
    if (!confirm(`Delete ${selected.email}? This cannot be undone.`)) return;
    void run("delete", () => api.del(`/users/${selected.id}`)).then((ok) => {
      if (ok) closePanel();
    });
  }

  function removeRow(u: UserWithRoles) {
    if (!confirm(`Delete ${u.email}? This cannot be undone.`)) return;
    void run("delete", () => api.del(`/users/${u.id}`)).then((ok) => {
      if (ok && selectedId === u.id) closePanel();
    });
  }

  const profileDirty =
    !!selected &&
    (nameDraft.trim() !== selected.display_name ||
      firstNameDraft.trim() !== (selected.first_name ?? "") ||
      middleNameDraft.trim() !== (selected.middle_name ?? "") ||
      lastNameDraft.trim() !== (selected.last_name ?? "") ||
      usernameDraft.trim() !== (selected.username ?? "") ||
      avatarDraft.trim() !== selected.avatar_url);

  function saveProfile() {
    if (!selected) return;
    void run("status", () =>
      api.patch(`/users/${selected.id}`, {
        display_name: nameDraft.trim(),
        first_name: firstNameDraft.trim(),
        middle_name: middleNameDraft.trim(),
        last_name: lastNameDraft.trim(),
        username: usernameDraft.trim(),
        avatar_url: avatarDraft.trim(),
      }),
    );
  }

  function resetProfile() {
    if (!selected) return;
    setNameDraft(selected.display_name);
    setFirstNameDraft(selected.first_name ?? "");
    setMiddleNameDraft(selected.middle_name ?? "");
    setLastNameDraft(selected.last_name ?? "");
    setUsernameDraft(selected.username ?? "");
    setAvatarDraft(selected.avatar_url);
    setPanelError(null);
  }

  return (
    <div className="space-y-5">
      <PageHeader
        title="Users"
        description="Accounts registered on the gateway. Select a user to manage their role and status."
        actions={
          <Button variant="ghost" onClick={() => void load()} loading={busy}>
            Refresh
          </Button>
        }
      />
      {error && <Alert>{error}</Alert>}
      {!users && !error && (
        <div className="grid place-items-center py-16">
          <Spinner />
        </div>
      )}
      {users && users.length === 0 && <EmptyState message="No users yet." />}
      {users && users.length > 0 && (
        <Table
          head={
            canWriteUsers
              ? ["Email", "Name", "Username", "Role", "Active", "Created", ""]
              : ["Email", "Name", "Username", "Role", "Active", "Created"]
          }
        >
          {users.map((u) => (
            <tr
              key={u.id}
              onClick={() => openPanel(u)}
              className={cn(
                "cursor-pointer transition-colors",
                u.id === selectedId
                  ? "bg-[var(--accent-soft)]"
                  : "hover:bg-[var(--sb-hover)]",
              )}
            >
              <Td className="font-medium text-[var(--app-fg)]">{u.email}</Td>
              <Td>{u.display_name || <span className="text-[var(--app-fg-faint)]">—</span>}</Td>
              <Td className="font-mono text-xs">{u.username || <span className="text-[var(--app-fg-faint)]">—</span>}</Td>
              <Td>
                <span className="flex flex-wrap items-center gap-1">
                  {u.roles.length === 0 && (
                    <span className="text-[var(--app-fg-faint)]">no role</span>
                  )}
                  {u.roles.map((name) => (
                    <Badge key={name} tone={name === "admin" ? "red" : "zinc"}>
                      {name}
                    </Badge>
                  ))}
                </span>
              </Td>
              <Td>
                <Badge tone={u.is_active ? "green" : "red"}>
                  {u.is_active ? "active" : "disabled"}
                </Badge>
              </Td>
              <Td className="text-[var(--app-fg-muted)]">
                {new Date(u.created_at + "Z").toLocaleString()}
              </Td>
              <Td className="text-right">
                <span className="flex items-center justify-end gap-2">
                  <span className="text-xs text-[var(--app-fg-faint)]">manage ›</span>
                  {canWriteUsers && (
                    <Button
                      variant="danger"
                      className="px-2.5 py-1 text-xs"
                      disabled={saving !== null}
                      onClick={(e) => {
                        e.stopPropagation();
                        removeRow(u);
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
        title={selected?.email ?? ""}
        subtitle={
          selected
            ? `registered ${new Date(selected.created_at + "Z").toLocaleString()}`
            : undefined
        }
      >
        {selected && (
          <>
            {panelError && <Alert>{panelError}</Alert>}

            <PanelSection label="Profile">
              <div className="flex items-center gap-3">
                <span className="grid h-11 w-11 shrink-0 place-items-center rounded-xl bg-gradient-to-br from-[#d63232] to-[#b91c1c] text-sm font-bold text-white">
                  {selected.email.charAt(0).toUpperCase()}
                </span>
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium text-[var(--app-fg)]">
                    {selected.email}
                  </p>
                  <p className="truncate text-xs text-[var(--app-fg-muted)]">
                    id {selected.id.slice(0, 8)}…
                  </p>
                </div>
              </div>
              {canWriteUsers ? (
                <>
                  <div className="grid grid-cols-2 gap-3">
                    <Field label="First name">
                      <Input
                        value={firstNameDraft}
                        disabled={saving !== null}
                        onChange={(e) => setFirstNameDraft(e.target.value)}
                      />
                    </Field>
                    <Field label="Last name">
                      <Input
                        value={lastNameDraft}
                        disabled={saving !== null}
                        onChange={(e) => setLastNameDraft(e.target.value)}
                      />
                    </Field>
                  </div>
                  <div className="grid grid-cols-2 gap-3">
                    <Field label="Middle name">
                      <Input
                        value={middleNameDraft}
                        disabled={saving !== null}
                        onChange={(e) => setMiddleNameDraft(e.target.value)}
                      />
                    </Field>
                    <Field label="Username">
                      <Input
                        value={usernameDraft}
                        disabled={saving !== null}
                        onChange={(e) => setUsernameDraft(e.target.value)}
                      />
                    </Field>
                  </div>
                  <Field label="Display name">
                    <Input
                      value={nameDraft}
                      disabled={saving !== null}
                      onChange={(e) => setNameDraft(e.target.value)}
                    />
                  </Field>
                  <Field label="Avatar url">
                    <Input
                      value={avatarDraft}
                      disabled={saving !== null}
                      placeholder="https://…"
                      onChange={(e) => setAvatarDraft(e.target.value)}
                    />
                  </Field>
                  <div className="flex gap-2">
                    <Button
                      className="flex-1"
                      disabled={!profileDirty}
                      loading={saving === "status"}
                      onClick={saveProfile}
                    >
                      Save
                    </Button>
                    <Button
                      variant="ghost"
                      className="flex-1"
                      disabled={!profileDirty || saving !== null}
                      onClick={resetProfile}
                    >
                      Cancel
                    </Button>
                  </div>
                  <Button
                    variant="ghost"
                    className="w-full"
                    loading={saving === "status"}
                    disabled={saving !== null && saving !== "status"}
                    onClick={toggleActive}
                  >
                    {selected.is_active ? "Disable account" : "Enable account"}
                  </Button>
                </>
              ) : (
                <dl className="mt-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
                  <dt className="text-[var(--app-fg-faint)]">First name</dt>
                  <dd className="text-[var(--app-fg)]">{selected.first_name || "—"}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Last name</dt>
                  <dd className="text-[var(--app-fg)]">{selected.last_name || "—"}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Username</dt>
                  <dd className="text-[var(--app-fg)]">{selected.username || "—"}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Display name</dt>
                  <dd className="text-[var(--app-fg)]">{selected.display_name || "—"}</dd>
                  <dt className="text-[var(--app-fg-faint)]">Status</dt>
                  <dd>
                    <Badge tone={selected.is_active ? "green" : "red"}>
                      {selected.is_active ? "active" : "disabled"}
                    </Badge>
                  </dd>
                </dl>
              )}
            </PanelSection>

            {canWriteUsers && roles && (
              <PanelSection label="Role">
                <p className="text-xs leading-relaxed text-[var(--app-fg-muted)]">
                  A user holds at most one role; picking a different one replaces it.
                  The role's permissions ride inside the token on next sign-in.
                </p>
                <div className="space-y-1.5">
                  <RoleChoice
                    label="No role"
                    description="Signed in, but nothing is allowed."
                    checked={roleChoice === ""}
                    disabled={saving === "role" || currentRole === ""}
                    onSelect={() => setRoleChoice("")}
                  />
                  {roles.map((r) => (
                    <RoleChoice
                      key={r.id}
                      label={r.name}
                      description={r.description}
                      checked={roleChoice === String(r.id)}
                      disabled={saving === "role" || currentRole === r.name}
                      onSelect={() => setRoleChoice(String(r.id))}
                    />
                  ))}
                </div>
                {(() => {
                  const activeRoleId = roleChoice || roles.find((r) => r.name === currentRole)?.id;
                  const activeRole = roles.find((r) => r.id === Number(activeRoleId));
                  if (!activeRole || activeRole.permissions.length === 0) return null;
                  return (
                    <div className="mt-3">
                      <p className="mb-1.5 text-xs font-medium text-[var(--app-fg-muted)]">
                        This role grants:
                      </p>
                      <div className="flex flex-wrap gap-1">
                        {activeRole.permissions.map((p) => (
                          <Badge key={p.id} tone="green">
                            {p.module}:{p.action}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  );
                })()}
                <div className="flex gap-2">
                  <Button
                    className="flex-1"
                    disabled={roleUnchanged}
                    loading={saving === "role"}
                    onClick={saveRole}
                  >
                    {roleChoice === "" ? "Revoke role" : "Apply role"}
                  </Button>
                  <Button
                    variant="ghost"
                    className="flex-1"
                    disabled={roleUnchanged || saving !== null}
                    onClick={() => {
                      setRoleChoice(currentRole);
                      setPanelError(null);
                    }}
                  >
                    Cancel
                  </Button>
                </div>
              </PanelSection>
            )}

            {canWriteUsers && (
              <PanelSection label="Danger zone">
                <Button
                  variant="danger"
                  className="w-full"
                  loading={saving === "delete"}
                  onClick={removeUser}
                >
                  Delete account
                </Button>
              </PanelSection>
            )}
          </>
        )}
      </SidePanel>
    </div>
  );
}

function RoleChoice({
  label,
  description,
  checked,
  disabled,
  onSelect,
}: {
  label: string;
  description: string;
  checked: boolean;
  disabled: boolean;
  onSelect: () => void;
}) {
  return (
    <label
      className={cn(
        "flex cursor-pointer items-start gap-3 rounded-xl border px-3 py-2.5 transition-colors",
        checked
          ? "border-[var(--accent)] bg-[var(--accent-soft)]"
          : "border-[var(--app-border)] hover:bg-[var(--sb-hover)]",
        disabled && "cursor-not-allowed opacity-50",
      )}
    >
      <input
        type="radio"
        name="role-choice"
        checked={checked}
        disabled={disabled}
        onChange={onSelect}
        className="mt-0.5 h-3.5 w-3.5 accent-[var(--accent)]"
      />
      <span className="min-w-0">
        <span className="block text-sm font-medium text-[var(--app-fg)]">{label}</span>
        <span className="block text-xs text-[var(--app-fg-muted)]">{description}</span>
      </span>
    </label>
  );
}
