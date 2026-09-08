import { useEffect, useState, useRef } from "react";
import { api } from "@gateway/lib";
import type { StoredFile } from "@gateway/lib";
import {
  Alert,
  Badge,
  Button,
  EmptyState,
  PageHeader,
  Spinner,
  Table,
  Td,
} from "@gateway/ui";

const DEFAULT_COLLECTIONS = ["maps", "avatars", "geojson", "files"];

export default function StoragePage() {
  const [collections, setCollections] = useState<string[]>(DEFAULT_COLLECTIONS);
  const [collection, setCollection] = useState<string>("maps");
  const [files, setFiles] = useState<StoredFile[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [uploading, setUploading] = useState(false);
  const [newTab, setNewTab] = useState(false);
  const [newName, setNewName] = useState("");
  const fileRef = useRef<HTMLInputElement>(null);

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const res = await api.get<{ files: StoredFile[] }>(`/storage/${collection}`);
      setFiles(res.files);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load files");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
  }, [collection]);

  async function doUpload() {
    const input = fileRef.current;
    if (!input?.files?.[0]) return;
    setUploading(true);
    setError(null);
    try {
      await api.upload(`/storage/${collection}`, input.files[0]);
      input.value = "";
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "upload failed");
    } finally {
      setUploading(false);
    }
  }

  async function remove(filename: string) {
    if (!confirm(`Delete ${filename}?`)) return;
    setBusy(true);
    try {
      await api.del(`/storage/${collection}/${encodeURIComponent(filename)}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    } finally {
      setBusy(false);
    }
  }

  function addCollection() {
    const name = newName.trim().toLowerCase().replace(/[^a-z0-9_-]/g, "");
    if (!name) return;
    if (!collections.includes(name)) {
      setCollections((prev) => [...prev, name]);
    }
    setCollection(name);
    setNewName("");
    setNewTab(false);
  }

  function fileUrl(f: StoredFile) {
    return `/api/v1/storage/${f.user_id}/${f.collection}/${encodeURIComponent(f.filename)}`;
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Storage"
        description="Manage uploaded files — avatars, GeoJSON data, and general assets."
      />

      {error && <Alert tone="error">{error}</Alert>}

      {/* ─── Collection tabs ─── */}
      <div className="flex items-center gap-0 border-b border-[var(--app-border)]">
        {collections.map((c) => (
          <button
            key={c}
            className={`px-4 py-2 text-sm font-medium transition-colors ${
              collection === c
                ? "border-b-2 border-[var(--accent)] text-[var(--app-fg)]"
                : "text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]"
            }`}
            onClick={() => setCollection(c)}
          >
            {c}
          </button>
        ))}

        {newTab ? (
          <div className="flex items-center gap-1 px-2 py-1">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") addCollection();
                if (e.key === "Escape") setNewTab(false);
              }}
              placeholder="name"
              className="w-28 rounded border border-[var(--app-border)] bg-[var(--app-card)] px-2 py-1 text-sm text-[var(--app-fg)]"
            />
            <Button variant="ghost" className="h-7 px-2 text-xs" onClick={addCollection}>
              Add
            </Button>
          </div>
        ) : (
          <button
            className="px-3 py-2 text-sm text-[var(--app-fg-muted)] hover:text-[var(--accent)]"
            onClick={() => setNewTab(true)}
          >
            +
          </button>
        )}
      </div>

      {/* ─── Toolbar ─── */}
      <div className="flex items-center justify-between">
        <span className="text-sm text-[var(--app-fg-muted)]">
          {files ? `${files.length} file${files.length !== 1 ? "s" : ""}` : ""}
        </span>

        <div className="flex items-center gap-2">
          <input
            ref={fileRef}
            type="file"
            className="hidden"
            onChange={doUpload}
          />
          <Button
            disabled={uploading}
            loading={uploading}
            onClick={() => fileRef.current?.click()}
          >
            Upload file
          </Button>
        </div>
      </div>

      {busy && !files && <Spinner />}

      {!busy && files && files.length === 0 && (
        <EmptyState message={`No files in ${collection} yet.`} />
      )}

      {files && files.length > 0 && (
        <Table head={["Filename", "Type", "Size", "Created", ""]}>
          {files.map((f) => (
            <tr key={f.id} className="hover:bg-[var(--app-card-2)]">
              <Td className="font-medium">
                <a
                  href={fileUrl(f)}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-[var(--accent)] hover:underline"
                >
                  {f.filename}
                </a>
              </Td>
              <Td>
                <Badge>{f.mime_type}</Badge>
              </Td>
              <Td className="text-sm text-[var(--app-fg-muted)]">
                {formatBytes(f.size_bytes)}
              </Td>
              <Td className="text-sm text-[var(--app-fg-muted)]">
                {new Date(f.created_at).toLocaleDateString()}
              </Td>
              <Td className="text-right">
                <button
                  className="text-sm text-red-500 hover:underline"
                  onClick={() => remove(f.filename)}
                >
                  Delete
                </button>
              </Td>
            </tr>
          ))}
        </Table>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}
