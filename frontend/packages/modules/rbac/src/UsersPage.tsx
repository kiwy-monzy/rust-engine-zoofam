import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import { Alert, Card, PageHeader, Spinner, Table } from "@gateway/ui";

type User = {
  id: string;
  email: string;
  display_name: string;
  is_active: boolean;
  created_at: string;
};

type Role = {
  id: number;
  name: string;
  description: string;
};

export function UsersPage({
  canWriteUsers,
  canReadRoles,
}: {
  canWriteUsers?: boolean;
  canReadRoles?: boolean;
}) {
  const [users, setUsers] = useState<User[]>([]);
  const [roles, setRoles] = useState<Role[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.get<{ users: User[] }>("/users").then((r) => r.users),
      canReadRoles ? api.get<{ roles: Role[] }>("/roles").then((r) => r.roles) : Promise.resolve([]),
    ])
      .then(([u, ro]) => {
        setUsers(u);
        setRoles(ro);
      })
      .catch((e) => setError(e instanceof Error ? e.message : "failed to load"))
      .finally(() => setLoading(false));
  }, [canReadRoles]);

  async function assignRole(userId: string, roleId: number) {
    await api.post(`/users/${userId}/roles/${roleId}`, {});
    const updated = await api.get<{ users: User[] }>("/users").then((r) => r.users);
    setUsers(updated);
  }

  if (loading) return <div className="grid place-items-center py-10"><Spinner /></div>;
  if (error) return <Alert>{error}</Alert>;

  return (
    <div className="space-y-6">
      <PageHeader title="Users" description="Manage user accounts and roles." />
      <Card>
        <Table
          head={["Email", "Display Name", "Active", "Created", ...(canWriteUsers && roles.length > 0 ? ["Actions"] : [])]}
        >
          {users.map((u) => (
            <tr key={u.id} className="border-b border-[var(--app-border)]">
              <td className="px-3 py-2">{u.email}</td>
              <td className="px-3 py-2">{u.display_name}</td>
              <td className="px-3 py-2">{u.is_active ? "Yes" : "No"}</td>
              <td className="px-3 py-2">{new Date(u.created_at).toLocaleDateString()}</td>
              {canWriteUsers && roles.length > 0 && (
                <td className="px-3 py-2">
                  <select
                    className="border rounded px-2 py-1 text-sm"
                    onChange={(e) => {
                      if (e.target.value) assignRole(u.id, Number(e.target.value));
                    }}
                    defaultValue=""
                  >
                    <option value="">Assign Role</option>
                    {roles.map((r) => (
                      <option key={r.id} value={r.id}>{r.name}</option>
                    ))}
                  </select>
                </td>
              )}
            </tr>
          ))}
        </Table>
      </Card>
    </div>
  );
}
