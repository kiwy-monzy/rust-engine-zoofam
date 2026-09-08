import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import { Alert, Button, Card, Input, PageHeader, Spinner, Table } from "@gateway/ui";

type Release = {
  id: number;
  version: string;
  platform: string;
  filename: string;
  file_size: number;
  sha256: string | null;
  changelog: string | null;
  download_count: number;
  created_at: string;
};

export function ReleasesPage({ canWriteReleases = false }: { canWriteReleases?: boolean }) {
  const [releases, setReleases] = useState<Release[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    api.get<{ releases: Release[] }>("/system/releases")
      .then((r) => setReleases(r.releases))
      .catch((e) => setError(e instanceof Error ? e.message : "failed to load releases"))
      .finally(() => setLoading(false));
  }, []);

  async function handleUpload(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setUploading(true);
    setError(null);

    const form = e.currentTarget;
    const fileInput = form.elements.namedItem("file") as HTMLInputElement;
    const versionInput = form.elements.namedItem("version") as HTMLInputElement;
    const platformInput = form.elements.namedItem("platform") as HTMLInputElement;
    const changelogInput = form.elements.namedItem("changelog") as HTMLInputElement;

    if (!fileInput.files?.[0] || !versionInput.value || !platformInput.value) {
      setError("File, version, and platform are required");
      setUploading(false);
      return;
    }

    const formData = new FormData();
    formData.append("file", fileInput.files[0]);
    formData.append("version", versionInput.value);
    formData.append("platform", platformInput.value);
    if (changelogInput.value) formData.append("changelog", changelogInput.value);

    try {
      await api.post("/system/releases", formData);
      const updated = await api.get<{ releases: Release[] }>("/system/releases").then((r) => r.releases);
      setReleases(updated);
      form.reset();
    } catch (e) {
      setError(e instanceof Error ? e.message : "upload failed");
    } finally {
      setUploading(false);
    }
  }

  async function handleDelete(id: number) {
    if (!confirm("Delete this release?")) return;
    try {
      await api.del(`/system/releases/${id}`);
      setReleases((prev) => prev.filter((r) => r.id !== id));
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  if (loading) return <div className="grid place-items-center py-10"><Spinner /></div>;
  if (error) return <Alert>{error}</Alert>;

  return (
    <div className="space-y-8">
      <PageHeader
        title="Releases"
        description="Manage application releases and updates."
      />

      {canWriteReleases && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold mb-4">Upload Release</h3>
          <form onSubmit={(e) => void handleUpload(e)} className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <div>
                <label className="text-sm font-medium">Version</label>
                <Input name="version" placeholder="1.0.0" className="mt-1" required />
              </div>
              <div>
                <label className="text-sm font-medium">Platform</label>
                <Input name="platform" placeholder="win64" className="mt-1" required />
              </div>
              <div>
                <label className="text-sm font-medium">Changelog</label>
                <Input name="changelog" placeholder="What's new..." className="mt-1" />
              </div>
            </div>
            <div className="flex items-end gap-4">
              <input type="file" name="file" accept="*/*" className="flex-1" required />
              <Button type="submit" loading={uploading}>Upload</Button>
            </div>
          </form>
        </Card>
      )}

      <Card>
        <Table head={["Version", "Platform", "Filename", "Size", "Downloads", "Created", ...(canWriteReleases ? ["Actions"] : [])]}>
          {releases.map((r) => (
            <tr key={r.id} className="border-b border-[var(--app-border)]">
              <td className="px-3 py-2">{r.version}</td>
              <td className="px-3 py-2">{r.platform}</td>
              <td className="px-3 py-2">{r.filename}</td>
              <td className="px-3 py-2">{formatBytes(r.file_size)}</td>
              <td className="px-3 py-2">{r.download_count}</td>
              <td className="px-3 py-2">{new Date(r.created_at).toLocaleDateString()}</td>
              {canWriteReleases && (
                <td className="px-3 py-2">
                  <Button variant="ghost" onClick={() => void handleDelete(r.id)}>Delete</Button>
                </td>
              )}
            </tr>
          ))}
        </Table>
      </Card>
    </div>
  );
}
