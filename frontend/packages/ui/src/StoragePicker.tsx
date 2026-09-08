import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import type { StoredFile } from "@gateway/lib";
import { Alert, Button, Card, EmptyState, Spinner } from "@gateway/ui";

export function StoragePicker({ collection, onPick, accept, multiple }: { collection: string; onPick: (f: StoredFile) => void; accept?: string; multiple?: boolean }) {
  const [files, setFiles] = useState<StoredFile[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [uploading, setUploading] = useState(false);
  async function load() { setBusy(true); setErr(null); try { const r = await api.get<{files: StoredFile[]}>(`/storage/${collection}`); setFiles(r.files); } catch(e){ setErr(e instanceof Error?e.message:String(e)); } finally { setBusy(false);} }
  useEffect(()=>{ void load(); },[collection]);
  async function onFile(e: React.ChangeEvent<HTMLInputElement>) {
    const f = e.target.files?.[0]; if(!f) return; setUploading(true);
    try { const r = await api.upload<{id:number; collection:string; filename:string; mime_type:string; size_bytes:number}>(`/storage/${collection}`, f); // reload and pick
      await load();
      // after reload, pick the newly uploaded if found
      const all = await api.get<{files: StoredFile[]}>(`/storage/${collection}`);
      const found = all.files.find(x=> x.filename===r.filename);
      if(found) onPick(found);
    } catch(e){ setErr(e instanceof Error?e.message:String(e)); } finally { setUploading(false); (e.target as HTMLInputElement).value=""; }
  }
  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs text-[var(--app-fg-muted)]">{collection} — {files? `${files.length} files` : ""}</span>
        <label className="cursor-pointer rounded-lg border border-[var(--app-border)] bg-[var(--app-card-2)] px-3 py-1.5 text-xs hover:bg-[var(--sb-hover)]">
          {uploading? "Uploading…":"Upload"}
          <input type="file" className="hidden" accept={accept} multiple={multiple} onChange={onFile} disabled={uploading} />
        </label>
      </div>
      {err && <Alert>{err}</Alert>}
      {busy && !files && <Spinner />}
      {!busy && files && files.length===0 && <EmptyState message={`No files in ${collection}. Upload one.`} />}
      {files && files.length>0 && (
        <div className="grid gap-2 grid-cols-2 sm:grid-cols-3">
          {files.map(f=> (
            <Card key={f.id} className="overflow-hidden cursor-pointer hover:shadow-md" onClick={()=> onPick(f)}>
              {f.mime_type.startsWith("image/") ? (
                <img src={`/api/v1/storage/${f.user_id}/${f.collection}/${encodeURIComponent(f.filename)}`} alt={f.filename} className="h-24 w-full object-cover" loading="lazy" />
              ) : (
                <div className="h-24 grid place-items-center text-xs text-[var(--app-fg-muted)]">{f.mime_type}</div>
              )}
              <div className="p-2">
                <div className="truncate text-xs font-medium">{f.filename}</div>
                <div className="text-[10px] text-[var(--app-fg-muted)]">{(f.size_bytes/1024).toFixed(1)} KB</div>
              </div>
            </Card>
          ))}
        </div>
      )}
      <Button variant="ghost" className="w-full text-xs" onClick={()=> void load()}>Refresh</Button>
    </div>
  );
}
