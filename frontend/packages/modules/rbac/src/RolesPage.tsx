import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import { Alert, Button, Card, Input, PageHeader, Spinner, Table } from "@gateway/ui";

type Role = {
  id: number;
  name: string;
  description: string;
};

type Permission = {
  id: number;
  module: string;
  action: string;
  description: string;
};

export function RolesPage({ canWriteRoles }: { canWriteRoles?: boolean }) {
  const [roles, setRoles] = useState<Role[]>([]);
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDesc, setNewDesc] = useState("");

  useEffect(() => {
    Promise.all([
      api.get<{ roles: Role[] }>("/roles").then((r) => r.roles),
      api.get<{ permissions: Permission[] }>("/permissions").then((r) => r.permissions),
    ])
      .then(([ro, pe]) => {
        setRoles(ro);
        setPermissions(pe);
      })
      .catch((e) => setError(e instanceof Error ? e.message : "failed to load"))
      .finally(() => setLoading(false));
  }, []);

  async function createRole() {
    await api.post("/roles", { name: newName, description: newDesc });
    const updated = await api.get<{ roles: Role[] }>("/roles").then((r) => r.roles);
    setRoles(updated);
    setShowCreate(false);
    setNewName("");
    setNewDesc("");
  }

  async function grant(roleId: number, permId: number) {
    await api.post(`/roles/${roleId}/permissions/${permId}`, {});
  }

  if (loading) return <div className="grid place-items-center py-10"><Spinner /></div>;
  if (error) return <Alert>{error}</Alert>;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Roles"
        description="Manage roles and their permissions."
        actions={
          canWriteRoles && (
            <Button onClick={() => setShowCreate(true)}>Create Role</Button>
          )
        }
      />
      {showCreate && (
        <Card className="p-4">
          <div className="flex gap-4 items-end">
            <div className="flex-1">
              <label className="text-sm font-medium">Name</label>
              <Input value={newName} onChange={(e) => setNewName(e.target.value)} className="mt-1" />
            </div>
            <div className="flex-1">
              <label className="text-sm font-medium">Description</label>
              <Input value={newDesc} onChange={(e) => setNewDesc(e.target.value)} className="mt-1" />
            </div>
            <Button onClick={createRole}>Save</Button>
            <Button variant="ghost" onClick={() => setShowCreate(false)}>Cancel</Button>
          </div>
        </Card>
      )}
      <Card>
        <Table
          head={["Name", "Description", "Permissions"]}
        >
          {roles.map((r) => (
            <tr key={r.id} className="border-b border-[var(--app-border)]">
              <td className="px-3 py-2">{r.name}</td>
              <td className="px-3 py-2">{r.description}</td>
              <td className="px-3 py-2">
                <select
                  className="border rounded px-2 py-1 text-sm"
                  onChange={(e) => {
                    if (e.target.value) grant(r.id, Number(e.target.value));
                  }}
                  defaultValue=""
                >
                  <option value="">Add Permission</option>
                  {permissions.map((p) => (
                    <option key={p.id} value={p.id}>{p.module}:{p.action}</option>
                  ))}
                </select>
              </td>
            </tr>
          ))}
        </Table>
      </Card>
    </div>
  );
}
