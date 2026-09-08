import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import { Alert, Button, Card, Input, PageHeader, Spinner, Table } from "@gateway/ui";

type Permission = {
  id: number;
  module: string;
  action: string;
  description: string;
};

export function PermissionsPage({ canWritePermissions }: { canWritePermissions?: boolean }) {
  const [permissions, setPermissions] = useState<Permission[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [newModule, setNewModule] = useState("");
  const [newAction, setNewAction] = useState("");
  const [newDesc, setNewDesc] = useState("");

  useEffect(() => {
    api.get<{ permissions: Permission[] }>("/permissions")
      .then((r) => setPermissions(r.permissions))
      .catch((e) => setError(e instanceof Error ? e.message : "failed to load"))
      .finally(() => setLoading(false));
  }, []);

  async function createPermission() {
    await api.post("/permissions", { module: newModule, action: newAction, description: newDesc });
    const updated = await api.get<{ permissions: Permission[] }>("/permissions").then((r) => r.permissions);
    setPermissions(updated);
    setShowCreate(false);
    setNewModule("");
    setNewAction("");
    setNewDesc("");
  }

  if (loading) return <div className="grid place-items-center py-10"><Spinner /></div>;
  if (error) return <Alert>{error}</Alert>;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Permissions"
        description="Define permissions for roles."
        actions={
          canWritePermissions && (
            <Button onClick={() => setShowCreate(true)}>Create Permission</Button>
          )
        }
      />
      {showCreate && (
        <Card className="p-4">
          <div className="flex gap-4 items-end">
            <div className="flex-1">
              <label className="text-sm font-medium">Module</label>
              <Input value={newModule} onChange={(e) => setNewModule(e.target.value)} className="mt-1" />
            </div>
            <div className="flex-1">
              <label className="text-sm font-medium">Action</label>
              <Input value={newAction} onChange={(e) => setNewAction(e.target.value)} className="mt-1" />
            </div>
            <div className="flex-1">
              <label className="text-sm font-medium">Description</label>
              <Input value={newDesc} onChange={(e) => setNewDesc(e.target.value)} className="mt-1" />
            </div>
            <Button onClick={createPermission}>Save</Button>
            <Button variant="ghost" onClick={() => setShowCreate(false)}>Cancel</Button>
          </div>
        </Card>
      )}
      <Card>
        <Table head={["Module", "Action", "Description"]}>
          {permissions.map((p) => (
            <tr key={p.id} className="border-b border-[var(--app-border)]">
              <td className="px-3 py-2">{p.module}</td>
              <td className="px-3 py-2">{p.action}</td>
              <td className="px-3 py-2">{p.description}</td>
            </tr>
          ))}
        </Table>
      </Card>
    </div>
  );
}
