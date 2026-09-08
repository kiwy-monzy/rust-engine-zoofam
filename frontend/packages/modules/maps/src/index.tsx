// Maps module — Sources/Layers/Features/Styles editor + Tiles management + Events log + Preview.
//
// Sub-tabs:
//   Edit    — Source / Layer / Style / Feature CRUD (existing tree + side-panel flow)
//   Tiles   — MapTile cache CRUD + map_tile_feature (per-feature tile index) CRUD
//   Events  — Map event log (view/filter/post/clear)
//   Preview — Leaflet preview with basemap + MVT/GeoJSON layers (existing)
import { useEffect, useMemo, useState } from "react";
import { api } from "@gateway/lib";
import type { StoredFile } from "@gateway/lib";
import { MapPreviewTab } from "./preview";
import {
  Alert,
  Badge,
  Button,
  Card,
  EmptyState,
  Field,
  Input,
  PageHeader,
  PanelSection,
  SidePanel,
  Spinner,
  Textarea,
  cn,
} from "@gateway/ui";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface MapSource {
  id: number;
  name: string;
  source_type: string;
  url: string | null;
  version: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}

interface MapLayer {
  id: number;
  source_id: number;
  name: string;
  layer_key: string;
  layer_type: string;
  description: string | null;
  min_zoom: number;
  max_zoom: number;
  z_index: number;
  visible: boolean;
  style_id: number | null;
  created_at: string;
  updated_at: string;
  feature_count?: number;
}

interface MapStyle {
  id: number;
  layer_id: number | null;
  name: string;
  style_type: string;
  definition: string;
  created_at: string;
  updated_at: string;
}

interface MapFeature {
  id: number;
  layer_id: number;
  feature_key: string;
  feature_type: string;
  geometry_type: string;
  properties: string | null;
  bbox_min_lon: number | null;
  bbox_min_lat: number | null;
  bbox_max_lon: number | null;
  bbox_max_lat: number | null;
  created_at: string;
  updated_at: string;
}

interface MapTile {
  id: number;
  layer_key: string;
  z: number;
  x: number;
  y: number;
  data: string; // base64 from server
  etag: string | null;
  created_at: string;
  updated_at: string;
}

interface MapTileFeature {
  id: number;
  feature_id: number;
  z: number;
  x: number;
  y: number;
}

interface MapEvent {
  id: number;
  event_type: string;
  lat: number | null;
  lon: number | null;
  zoom: number | null;
  layer_key: string | null;
  feature_key: string | null;
  data: string | null;
  created_at: string;
}

type Tab = "edit" | "tiles" | "events" | "preview";

function slugify(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

// ---------------------------------------------------------------------------
// MapsPage
// ---------------------------------------------------------------------------

export function MapsPage({ canWriteMaps = false }: { canWriteMaps?: boolean }) {
  const [tab, setTab] = useState<Tab>("edit");

  return (
    <div className="flex h-[calc(100vh-8rem)] flex-col">
      <PageHeader
        title="Maps"
        description="Sources → Layers → Features → Tiles — full CRUD plus event log and live preview."
      />

      <div className="flex items-center gap-1 border-b border-[var(--app-border)]">
        {(["edit", "tiles", "events", "preview"] as Tab[]).map((t) => (
          <button
            key={t}
            type="button"
            onClick={() => setTab(t)}
            className={cn(
              "relative px-3 py-2 text-sm font-medium transition-colors",
              tab === t
                ? "border-b-2 border-[var(--accent)] text-[var(--app-fg)]"
                : "text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]",
            )}
          >
            {t === "edit" ? "Edit" : t === "tiles" ? "Tiles" : t === "events" ? "Events" : "Preview"}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-hidden">
        {tab === "edit" && <EditTab canWriteMaps={canWriteMaps} />}
        {tab === "tiles" && <TilesTab canWriteMaps={canWriteMaps} />}
        {tab === "events" && <EventsTab canWriteMaps={canWriteMaps} />}
        {tab === "preview" && <MapPreviewTab canWrite={canWriteMaps} />}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// EDIT TAB — sources / layers / styles / features
// ---------------------------------------------------------------------------

function EditTab({ canWriteMaps }: { canWriteMaps: boolean }) {
  const [sources, setSources] = useState<MapSource[]>([]);
  const [layers, setLayers] = useState<MapLayer[]>([]);
  const [styles, setStyles] = useState<MapStyle[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const [expandedSourceIds, setExpandedSourceIds] = useState<Set<number>>(new Set());
  const [selectedLayerId, setSelectedLayerId] = useState<number | null>(null);

  // SidePanel
  const [panelOpen, setPanelOpen] = useState(false);
  const [panelMode, setPanelMode] = useState<
    "create-source" | "edit-source" | "create-layer" | "edit-layer" | "create-style" | "edit-style" | "create-feature" | "edit-feature"
  >("edit-source");
  const [draft, setDraft] = useState<Record<string, unknown>>({});
  const [saving, setSaving] = useState(false);
  const [panelError, setPanelError] = useState<string | null>(null);

  // File picker for source URL
  const [storedFiles, setStoredFiles] = useState<StoredFile[] | null>(null);
  const [showFilePicker, setShowFilePicker] = useState(false);

  // GeoJSON import
  const [importOpen, setImportOpen] = useState(false);
  const [importLayerId, setImportLayerId] = useState<number | "">("");
  const [importFileType, setImportFileType] = useState("feature");
  const [importFile, setImportFile] = useState<File | null>(null);
  const [importing, setImporting] = useState(false);
  const [importResult, setImportResult] = useState<string | null>(null);

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const [s, l, st] = await Promise.all([
        api.get<{ sources: MapSource[] }>("/map/sources"),
        api.get<{ layers: MapLayer[] }>("/map/layers"),
        api.get<{ styles: MapStyle[] }>("/map/styles"),
      ]);
      setSources(s.sources);
      setLayers(l.layers);
      setStyles(st.styles);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load");
    } finally {
      setBusy(false);
    }
  }
  useEffect(() => {
    void load();
  }, []);

  const layersForSource = (sid: number) => layers.filter((l) => l.source_id === sid);
  const stylesForLayer = (lid: number) => styles.filter((s) => s.layer_id === lid);
  const selectedLayer = selectedLayerId !== null ? layers.find((l) => l.id === selectedLayerId) : null;

  function toggleSource(id: number) {
    setExpandedSourceIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  async function removeLayer(id: number) {
    const name = layers.find((l) => l.id === id)?.name ?? "this layer";
    const styleCount = stylesForLayer(id).length;
    const msg =
      styleCount > 0
        ? `Delete "${name}"? This will also remove ${styleCount} style(s) and all features/tiles.`
        : `Delete "${name}"? This will also remove all features and tiles.`;
    if (!confirm(msg)) return;
    try {
      await api.del(`/map/layers/${id}`);
      setSelectedLayerId(null);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }
  async function removeSource(id: number) {
    if (!confirm("Delete this source? All its layers/styles/features/tiles will be removed too.")) return;
    try {
      await api.del(`/map/sources/${id}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }
  async function removeStyle(id: number) {
    if (!confirm("Delete this style?")) return;
    try {
      await api.del(`/map/styles/${id}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }
  async function toggleVisibility(layer: MapLayer) {
    try {
      await api.post(`/map/layers/${layer.id}/toggle-visibility`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "toggle failed");
    }
  }

  function openEditSource(s: MapSource) {
    setPanelMode("edit-source");
    setDraft({ ...s });
    setPanelOpen(true);
    setPanelError(null);
    setStoredFiles(null);
    setShowFilePicker(false);
  }
  function openCreateLayer(sourceId: number) {
    setPanelMode("create-layer");
    setDraft({
      name: "",
      layer_key: "",
      layer_type: "vector",
      source_id: sourceId,
      min_zoom: 0,
      max_zoom: 22,
      z_index: 0,
      visible: true,
      description: "",
    });
    setPanelOpen(true);
    setPanelError(null);
  }
  function openCreateStyle(layerId: number | null) {
    setPanelMode("create-style");
    setDraft({ name: "", style_type: "vector", definition: "{}", layer_id: layerId });
    setPanelOpen(true);
    setPanelError(null);
  }
  function openEditStyle(s: MapStyle) {
    setPanelMode("edit-style");
    setDraft({ ...s });
    setPanelOpen(true);
    setPanelError(null);
  }
  function openCreateFeature(_layerId: number) {
    setPanelMode("create-feature");
    setDraft({
      feature_key: "",
      feature_type: "feature",
      geometry_type: "Point",
      geometry: "",
      properties: "{}",
    });
    setPanelOpen(true);
    setPanelError(null);
  }
  function closePanel() {
    setPanelOpen(false);
    setPanelError(null);
    setStoredFiles(null);
    setShowFilePicker(false);
  }

  async function save() {
    setSaving(true);
    setPanelError(null);
    try {
      if (panelMode === "create-source") {
        await api.post("/map/sources", {
          name: draft.name,
          source_type: draft.source_type,
          url: draft.url || null,
          version: draft.version || null,
          description: draft.description || null,
        });
      } else if (panelMode === "edit-source") {
        await api.patch(`/map/sources/${draft.id}`, {
          name: draft.name,
          source_type: draft.source_type,
          url: draft.url || null,
          version: draft.version || null,
          description: draft.description || null,
        });
      } else if (panelMode === "create-layer") {
        await api.post("/map/layers", {
          name: draft.name,
          layer_key: draft.layer_key,
          layer_type: draft.layer_type,
          source_id: draft.source_id,
          min_zoom: draft.min_zoom ?? 0,
          max_zoom: draft.max_zoom ?? 22,
          z_index: draft.z_index ?? 0,
          visible: draft.visible ?? true,
          description: draft.description || null,
          style_id: draft.style_id ?? null,
        });
      } else if (panelMode === "edit-layer") {
        await api.patch(`/map/layers/${draft.id}`, {
          name: draft.name,
          layer_key: draft.layer_key,
          layer_type: draft.layer_type,
          source_id: draft.source_id,
          min_zoom: draft.min_zoom ?? 0,
          max_zoom: draft.max_zoom ?? 22,
          z_index: draft.z_index ?? 0,
          visible: draft.visible ?? true,
          description: draft.description || null,
          style_id: draft.style_id ?? null,
        });
      } else if (panelMode === "create-style") {
        await api.post("/map/styles", {
          name: draft.name,
          style_type: draft.style_type,
          definition: draft.definition,
          layer_id: draft.layer_id ?? null,
        });
      } else if (panelMode === "edit-style") {
        await api.patch(`/map/styles/${draft.id}`, {
          name: draft.name,
          style_type: draft.style_type,
          definition: draft.definition,
          layer_id: draft.layer_id ?? null,
        });
      } else if (panelMode === "create-feature") {
        await api.post(`/map/features`, {
          layer_id: selectedLayerId,
          feature_key: draft.feature_key,
          feature_type: draft.feature_type,
          geometry_type: draft.geometry_type,
          geometry: draft.geometry,
          properties: draft.properties,
        });
      } else if (panelMode === "edit-feature") {
        await api.patch(`/map/features/${draft.id}`, {
          feature_key: draft.feature_key,
          feature_type: draft.feature_type,
          geometry_type: draft.geometry_type,
          geometry: draft.geometry,
          properties: draft.properties,
        });
      }
      closePanel();
      await load();
    } catch (e) {
      setPanelError(e instanceof Error ? e.message : "save failed");
    } finally {
      setSaving(false);
    }
  }

  async function loadStoredFiles() {
    try {
      const res = await api.get<{ files: StoredFile[] }>("/storage/geojson");
      setStoredFiles(res.files);
      setShowFilePicker(true);
    } catch {
      setStoredFiles([]);
      setShowFilePicker(true);
    }
  }

  async function doImport() {
    if (!importFile || importLayerId === "") return;
    setImporting(true);
    setImportResult(null);
    try {
      const res = await api.upload<{ imported: number; errors: string[] }>(
        `/map/layers/${importLayerId}/import`,
        importFile,
        { feature_type: importFileType },
      );
      setImportResult(
        res.errors.length > 0 ? `Imported ${res.imported} (${res.errors.length} errors)` : `Imported ${res.imported} features`,
      );
      await load();
    } catch (e) {
      setImportResult(e instanceof Error ? e.message : "import failed");
    } finally {
      setImporting(false);
    }
  }

  return (
    <div className="flex h-full gap-4 overflow-hidden">
      {/* LEFT: source → layer tree */}
      <div className="w-80 shrink-0 overflow-y-auto rounded-lg border border-[var(--app-border)] bg-[var(--app-card)]">
        <div className="sticky top-0 z-10 flex items-center justify-between border-b border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2">
          <span className="text-xs font-medium uppercase text-[var(--app-fg-muted)]">Sources</span>
          {busy && <Spinner />}
        </div>
        {!busy && sources.length === 0 && (
          <div className="p-4">
            <EmptyState message="No sources yet." />
          </div>
        )}
        {sources.map((src) => {
          const srcLayers = layersForSource(src.id);
          const isExpanded = expandedSourceIds.has(src.id);
          return (
            <div key={src.id} className="border-b border-[var(--app-border)] last:border-b-0">
              <div
                className="flex cursor-pointer items-center gap-2 px-3 py-2 hover:bg-[var(--app-card-2)] transition-colors"
                onClick={() => toggleSource(src.id)}
              >
                <span
                  className={cn(
                    "w-4 text-center text-xs text-[var(--app-fg-muted)] transition-transform",
                    isExpanded && "rotate-90",
                  )}
                >
                  ▸
                </span>
                <span className="flex-1 truncate text-sm font-medium text-[var(--app-fg)]">{src.name}</span>
                <Badge>{srcLayers.length}</Badge>
                {canWriteMaps && (
                  <>
                    <button
                      className="ml-1 rounded p-0.5 text-[var(--app-fg-muted)] hover:bg-[var(--app-card)] hover:text-[var(--app-fg)]"
                      title="Edit source"
                      onClick={(e) => {
                        e.stopPropagation();
                        openEditSource(src);
                      }}
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                      </svg>
                    </button>
                    <button
                      className="ml-1 rounded p-0.5 text-[var(--app-fg-muted)] hover:bg-[var(--app-card)] hover:text-[var(--app-fg)]"
                      title="Delete source"
                      onClick={(e) => {
                        e.stopPropagation();
                        void removeSource(src.id);
                      }}
                    >
                      <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2m3 0v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6h14z" />
                      </svg>
                    </button>
                  </>
                )}
              </div>
              {isExpanded && (
                <div className="bg-[var(--app-bg)]">
                  {srcLayers.length === 0 && <div className="px-6 py-2 text-xs text-[var(--app-fg-muted)]">No layers</div>}
                  {srcLayers.map((layer) => {
                    const layerStyles = stylesForLayer(layer.id);
                    const isSelected = selectedLayerId === layer.id;
                    return (
                      <div
                        key={layer.id}
                        className={cn(
                          "flex cursor-pointer items-center gap-2 px-6 py-2 transition-colors border-l-2",
                          isSelected
                            ? "bg-[var(--accent-soft)] border-l-[var(--accent)]"
                            : "border-l-transparent hover:bg-[var(--app-card-2)]",
                        )}
                        onClick={() => setSelectedLayerId(layer.id)}
                      >
                        <button
                          className="w-4 text-center text-xs text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]"
                          onClick={(e) => {
                            e.stopPropagation();
                            void toggleVisibility(layer);
                          }}
                          title={layer.visible ? "Visible — click to hide" : "Hidden — click to show"}
                        >
                          {layer.visible ? "◉" : "○"}
                        </button>
                        <div className="flex-1 min-w-0">
                          <div className="truncate text-sm text-[var(--app-fg)]">{layer.name}</div>
                          <div className="truncate text-[10px] text-[var(--app-fg-muted)]">
                            {layer.layer_key} · z{layer.z_index}
                          </div>
                        </div>
                        {layerStyles.length > 0 && <Badge>{layerStyles.length} style</Badge>}
                      </div>
                    );
                  })}
                  {canWriteMaps && (
                    <button
                      className="w-full px-6 py-1.5 text-left text-xs text-[var(--app-fg-muted)] hover:bg-[var(--app-card-2)] hover:text-[var(--app-fg)]"
                      onClick={() => openCreateLayer(src.id)}
                    >
                      + add layer
                    </button>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* RIGHT: layer detail */}
      <div className="flex flex-1 flex-col overflow-hidden rounded-lg border border-[var(--app-border)]">
        <div className="flex items-center justify-between border-b border-[var(--app-border)] px-4 py-2">
          <span className="text-sm font-medium">{selectedLayer ? selectedLayer.name : "Layer detail"}</span>
          {selectedLayer && canWriteMaps && (
            <div className="flex gap-1">
              <Button variant="ghost" className="h-7 px-2 text-xs" onClick={() => setImportOpen(true)}>
                Import GeoJSON
              </Button>
              <Button variant="ghost" className="h-7 px-2 text-xs" onClick={() => openCreateFeature(selectedLayer.id)}>
                + Feature
              </Button>
              <Button variant="ghost" className="h-7 px-2 text-xs" onClick={() => openCreateStyle(selectedLayer.id)}>
                + Style
              </Button>
              <Button variant="danger" className="h-7 px-2 text-xs" onClick={() => void removeLayer(selectedLayer.id)}>
                Delete
              </Button>
            </div>
          )}
        </div>
        <div className="flex-1 overflow-auto">
          {error && <Alert tone="error">{error}</Alert>}
          {!selectedLayer ? (
            <div className="flex h-full items-center justify-center text-sm text-[var(--app-fg-muted)]">
              Select a layer from the source list.
            </div>
          ) : (
            <LayerDetail
              layer={selectedLayer}
              styles={stylesForLayer(selectedLayer.id)}
              onEditStyle={openEditStyle}
              onRemoveStyle={(s) => void removeStyle(s.id)}
              onAddFeature={() => openCreateFeature(selectedLayer.id)}
            />
          )}
        </div>
      </div>

      <SidePanel
        open={panelOpen}
        title={panelMode.startsWith("create") ? `New ${panelMode.split("-")[1]}` : `Edit: ${draft.name ?? draft.feature_key ?? ""}`}
        onClose={closePanel}
      >
        {panelError && <Alert tone="error">{panelError}</Alert>}
        {(panelMode === "create-source" || panelMode === "edit-source") && (
          <SourceForm
            draft={draft}
            setDraft={setDraft}
            canWrite={canWriteMaps}
            loadStoredFiles={loadStoredFiles}
            showFilePicker={showFilePicker}
            setShowFilePicker={setShowFilePicker}
            storedFiles={storedFiles}
          />
        )}
        {(panelMode === "create-layer" || panelMode === "edit-layer") && (
          <LayerForm draft={draft} setDraft={setDraft} sources={sources} />
        )}
        {(panelMode === "create-style" || panelMode === "edit-style") && (
          <StyleForm draft={draft} setDraft={setDraft} layers={layers} />
        )}
        {(panelMode === "create-feature" || panelMode === "edit-feature") && (
          <FeatureForm draft={draft} setDraft={setDraft} />
        )}
        <div className="flex justify-end gap-2 pt-4">
          <Button variant="ghost" onClick={closePanel}>
            Cancel
          </Button>
          <Button onClick={save} loading={saving}>
            Save
          </Button>
        </div>
      </SidePanel>

      <SidePanel
        open={importOpen}
        title="Import GeoJSON"
        onClose={() => {
          setImportOpen(false);
          setImportResult(null);
          setImportFile(null);
        }}
      >
        <PanelSection label="File">
          <Field label="Layer">
            <select
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
              value={importLayerId}
              onChange={(e) => setImportLayerId(e.target.value ? Number(e.target.value) : "")}
            >
              <option value="">Select a layer…</option>
              {layers.map((l) => (
                <option key={l.id} value={l.id}>
                  {l.name} ({l.layer_key})
                </option>
              ))}
            </select>
          </Field>
          <Field label="Feature type">
            <Input
              value={importFileType}
              onChange={(e) => setImportFileType(e.target.value)}
              placeholder="e.g. building"
            />
          </Field>
          <Field label="GeoJSON file">
            <input
              type="file"
              accept=".geojson,application/geo+json,application/json"
              className="w-full text-sm text-[var(--app-fg-muted)] file:mr-4 file:rounded-lg file:border-0 file:bg-[var(--accent)] file:px-3 file:py-1.5 file:text-sm file:font-medium file:text-white hover:file:opacity-90"
              onChange={(e) => setImportFile(e.target.files?.[0] ?? null)}
            />
          </Field>
          {importResult && (
            <Alert tone={importResult.includes("failed") || importResult.includes("errors") ? "error" : "success"}>
              {importResult}
            </Alert>
          )}
        </PanelSection>
        <div className="flex justify-end gap-2 pt-4">
          <Button
            variant="ghost"
            onClick={() => {
              setImportOpen(false);
              setImportResult(null);
              setImportFile(null);
            }}
          >
            Cancel
          </Button>
          <Button disabled={importLayerId === "" || !importFile || importing} loading={importing} onClick={doImport}>
            Import
          </Button>
        </div>
      </SidePanel>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Layer detail (right panel) — also shows features and styles
// ---------------------------------------------------------------------------

function LayerDetail({
  layer,
  styles,
  onEditStyle,
  onRemoveStyle,
  onAddFeature,
}: {
  layer: MapLayer;
  styles: MapStyle[];
  onEditStyle: (s: MapStyle) => void;
  onRemoveStyle: (s: MapStyle) => void;
  onAddFeature: () => void;
}) {
  const [features, setFeatures] = useState<MapFeature[] | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    api
      .get<{ features: MapFeature[] }>(`/map/features/layer/${layer.id}`)
      .then((r) => setFeatures(r.features))
      .finally(() => setLoading(false));
  }, [layer.id]);

  return (
    <div className="p-4 space-y-4">
      <div className="grid grid-cols-2 gap-3 text-sm">
        <div>
          <span className="text-[var(--app-fg-muted)]">Type:</span> {layer.layer_type}
        </div>
        <div>
          <span className="text-[var(--app-fg-muted)]">Zoom:</span> {layer.min_zoom}–{layer.max_zoom}
        </div>
        <div>
          <span className="text-[var(--app-fg-muted)]">Z-index:</span> {layer.z_index}
        </div>
        <div>
          <span className="text-[var(--app-fg-muted)]">Visible:</span>{" "}
          {layer.visible ? <Badge>on</Badge> : <Badge>off</Badge>}
        </div>
        <div className="col-span-2">
          <span className="text-[var(--app-fg-muted)]">Layer key:</span>{" "}
          <code className="rounded bg-[var(--accent-soft)] px-1.5 py-0.5 text-xs">{layer.layer_key}</code>
        </div>
      </div>

      <div>
        <div className="mb-1 flex items-center justify-between">
          <p className="text-xs font-medium text-[var(--app-fg-muted)]">
            Features ({features?.length ?? 0})
          </p>
          <button
            className="text-[10px] uppercase text-[var(--accent-fg)] hover:underline"
            onClick={onAddFeature}
          >
            + add feature
          </button>
        </div>
        {loading ? (
          <Spinner />
        ) : !features || features.length === 0 ? (
          <EmptyState message="No features." />
        ) : (
          <div className="max-h-60 overflow-y-auto rounded border border-[var(--app-border)]">
            <table className="w-full text-left text-xs">
              <thead className="bg-[var(--app-card-2)] text-[var(--app-fg-muted)]">
                <tr>
                  <th className="px-2 py-1">Key</th>
                  <th className="px-2 py-1">Type</th>
                  <th className="px-2 py-1">Geometry</th>
                </tr>
              </thead>
              <tbody>
                {features.slice(0, 50).map((f) => (
                  <tr key={f.id} className="border-t border-[var(--app-border)]">
                    <td className="px-2 py-1 font-mono">{f.feature_key}</td>
                    <td className="px-2 py-1">{f.feature_type}</td>
                    <td className="px-2 py-1">
                      <Badge>{f.geometry_type}</Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {features.length > 50 && (
              <div className="border-t border-[var(--app-border)] px-2 py-1 text-[10px] text-[var(--app-fg-muted)]">
                + {features.length - 50} more
              </div>
            )}
          </div>
        )}
      </div>

      <div>
        <p className="mb-1 text-xs font-medium text-[var(--app-fg-muted)]">Styles ({styles.length})</p>
        {styles.length === 0 ? (
          <EmptyState message="No styles yet." />
        ) : (
          styles.map((s) => (
            <div
              key={s.id}
              className="flex items-center justify-between gap-2 rounded px-2 py-1 text-xs hover:bg-[var(--app-card-2)]"
            >
              <button
                className="flex flex-1 items-center gap-2 text-left"
                onClick={() => onEditStyle(s)}
              >
                <span className="text-[var(--app-fg-muted)]">🎨</span>
                <span className="font-medium">{s.name}</span>
                <Badge>{s.style_type}</Badge>
              </button>
              <button
                className="rounded px-1 text-red-500 hover:bg-red-500/10"
                onClick={() => onRemoveStyle(s)}
                title="Delete style"
              >
                ×
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// TILES TAB — full tile cache + per-feature tile index CRUD
// ---------------------------------------------------------------------------

function TilesTab({ canWriteMaps }: { canWriteMaps: boolean }) {
  const [tiles, setTiles] = useState<MapTile[] | null>(null);
  const [tileFeatures, setTileFeatures] = useState<MapTileFeature[] | null>(null);
  const [filterLayer, setFilterLayer] = useState<string>("");
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tfOpen, setTfOpen] = useState(false);
  const [tfDraft, setTfDraft] = useState<Record<string, unknown>>({});
  const layers = useList<MapLayer>("/map/layers", "layers");

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const qs = filterLayer ? `?layer_key=${encodeURIComponent(filterLayer)}` : "";
      const [t, tf] = await Promise.all([
        api.get<{ tiles: MapTile[] }>(`/map/tiles${qs}`),
        api.get<{ tile_features: MapTileFeature[] }>(`/map/tile-features`),
      ]);
      setTiles(t.tiles);
      setTileFeatures(tf.tile_features);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load");
    } finally {
      setBusy(false);
    }
  }
  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filterLayer]);

  async function clearLayer(layerKey: string) {
    if (!confirm(`Clear all tiles cached for "${layerKey}"?`)) return;
    try {
      const r = await api.del<{ cleared: number }>(`/map/tiles/layer/${encodeURIComponent(layerKey)}/clear`);
      await load();
      alert(`Cleared ${r.cleared} tile(s).`);
    } catch (e) {
      setError(e instanceof Error ? e.message : "clear failed");
    }
  }
  async function deleteTile(layerKey: string, z: number, x: number, y: number) {
    if (!confirm(`Delete tile ${z}/${x}/${y} for "${layerKey}"?`)) return;
    try {
      await api.del(`/map/tiles/${encodeURIComponent(layerKey)}/${z}/${x}/${y}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }
  async function removeTileFeature(id: number) {
    if (!confirm("Delete this tile-feature index entry?")) return;
    try {
      await api.del(`/map/tile-features/${id}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }
  async function addTileFeature() {
    setTfDraft({ feature_id: "", z: 0, x: 0, y: 0 });
    setTfOpen(true);
  }
  async function saveTileFeature() {
    try {
      await api.post(`/map/tile-features`, {
        feature_id: Number(tfDraft.feature_id),
        z: Number(tfDraft.z),
        x: Number(tfDraft.x),
        y: Number(tfDraft.y),
      });
      setTfOpen(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "save failed");
    }
  }

  const layerKeys = useMemo(() => {
    const s = new Set<string>();
    (tiles ?? []).forEach((t) => s.add(t.layer_key));
    (layers.data ?? []).forEach((l) => s.add(l.layer_key));
    return Array.from(s).sort();
  }, [tiles, layers.data]);

  return (
    <div className="grid h-full grid-cols-1 gap-4 overflow-hidden p-4 lg:grid-cols-2">
      <Card className="flex flex-col overflow-hidden p-4">
        <div className="mb-3 flex items-center justify-between gap-2">
          <h3 className="text-sm font-semibold">Tile cache</h3>
          <div className="flex gap-2">
            <select
              className="rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-2 py-1 text-xs"
              value={filterLayer}
              onChange={(e) => setFilterLayer(e.target.value)}
            >
              <option value="">All layers</option>
              {layerKeys.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
            {canWriteMaps && filterLayer && (
              <Button variant="danger" className="h-7 px-2 text-xs" onClick={() => void clearLayer(filterLayer)}>
                Clear layer
              </Button>
            )}
            <Button variant="ghost" className="h-7 px-2 text-xs" onClick={() => void load()}>
              Refresh
            </Button>
          </div>
        </div>
        {error && <Alert tone="error">{error}</Alert>}
        {busy ? (
          <Spinner />
        ) : !tiles || tiles.length === 0 ? (
          <EmptyState message="No tiles cached yet." />
        ) : (
          <div className="flex-1 overflow-y-auto rounded border border-[var(--app-border)]">
            <table className="w-full text-left text-xs">
              <thead className="sticky top-0 bg-[var(--app-card-2)] text-[var(--app-fg-muted)]">
                <tr>
                  <th className="px-2 py-1.5">Layer</th>
                  <th className="px-2 py-1.5">z/x/y</th>
                  <th className="px-2 py-1.5">Size</th>
                  <th className="px-2 py-1.5"></th>
                </tr>
              </thead>
              <tbody>
                {tiles.map((t) => (
                  <tr key={t.id} className="border-t border-[var(--app-border)]">
                    <td className="px-2 py-1 font-mono">{t.layer_key}</td>
                    <td className="px-2 py-1 font-mono">
                      {t.z}/{t.x}/{t.y}
                    </td>
                    <td className="px-2 py-1">{Math.round((t.data?.length ?? 0) * 0.75)} B</td>
                    <td className="px-2 py-1 text-right">
                      {canWriteMaps && (
                        <button
                          className="rounded px-1 text-red-500 hover:bg-red-500/10"
                          onClick={() => void deleteTile(t.layer_key, t.z, t.x, t.y)}
                          title="Delete tile"
                        >
                          ×
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <Card className="flex flex-col overflow-hidden p-4">
        <div className="mb-3 flex items-center justify-between">
          <h3 className="text-sm font-semibold">Per-feature tile index</h3>
          {canWriteMaps && (
            <Button className="h-7 px-2 text-xs" onClick={addTileFeature}>
              + Index entry
            </Button>
          )}
        </div>
        {busy ? (
          <Spinner />
        ) : !tileFeatures || tileFeatures.length === 0 ? (
          <EmptyState message="No index entries." />
        ) : (
          <div className="flex-1 overflow-y-auto rounded border border-[var(--app-border)]">
            <table className="w-full text-left text-xs">
              <thead className="sticky top-0 bg-[var(--app-card-2)] text-[var(--app-fg-muted)]">
                <tr>
                  <th className="px-2 py-1.5">ID</th>
                  <th className="px-2 py-1.5">Feature</th>
                  <th className="px-2 py-1.5">Tile (z/x/y)</th>
                  <th className="px-2 py-1.5"></th>
                </tr>
              </thead>
              <tbody>
                {tileFeatures.map((tf) => (
                  <tr key={tf.id} className="border-t border-[var(--app-border)]">
                    <td className="px-2 py-1 font-mono">{tf.id}</td>
                    <td className="px-2 py-1 font-mono">{tf.feature_id}</td>
                    <td className="px-2 py-1 font-mono">
                      {tf.z}/{tf.x}/{tf.y}
                    </td>
                    <td className="px-2 py-1 text-right">
                      {canWriteMaps && (
                        <button
                          className="rounded px-1 text-red-500 hover:bg-red-500/10"
                          onClick={() => void removeTileFeature(tf.id)}
                        >
                          ×
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <SidePanel open={tfOpen} onClose={() => setTfOpen(false)} title="New tile-feature index entry">
        <PanelSection label="Index entry">
          <Field label="Feature ID">
            <Input
              type="number"
              value={(tfDraft.feature_id as number) ?? ""}
              onChange={(e) => setTfDraft((p) => ({ ...p, feature_id: e.target.value }))}
            />
          </Field>
          <div className="grid grid-cols-3 gap-2">
            <Field label="z">
              <Input
                type="number"
                value={(tfDraft.z as number) ?? 0}
                onChange={(e) => setTfDraft((p) => ({ ...p, z: e.target.value }))}
              />
            </Field>
            <Field label="x">
              <Input
                type="number"
                value={(tfDraft.x as number) ?? 0}
                onChange={(e) => setTfDraft((p) => ({ ...p, x: e.target.value }))}
              />
            </Field>
            <Field label="y">
              <Input
                type="number"
                value={(tfDraft.y as number) ?? 0}
                onChange={(e) => setTfDraft((p) => ({ ...p, y: e.target.value }))}
              />
            </Field>
          </div>
        </PanelSection>
        <div className="flex justify-end gap-2 pt-4">
          <Button variant="ghost" onClick={() => setTfOpen(false)}>
            Cancel
          </Button>
          <Button onClick={saveTileFeature}>Save</Button>
        </div>
      </SidePanel>
    </div>
  );
}

// ---------------------------------------------------------------------------
// EVENTS TAB — map event log
// ---------------------------------------------------------------------------

function EventsTab({ canWriteMaps }: { canWriteMaps: boolean }) {
  const [events, setEvents] = useState<MapEvent[] | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterType, setFilterType] = useState("");
  const [filterLayer, setFilterLayer] = useState("");
  const [postOpen, setPostOpen] = useState(false);
  const [draft, setDraft] = useState<Record<string, unknown>>({
    event_type: "click",
    lat: null,
    lon: null,
    zoom: null,
    layer_key: "",
    feature_key: "",
    data: "",
  });
  const [posting, setPosting] = useState(false);

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      if (filterType) params.set("event_type", filterType);
      if (filterLayer) params.set("layer_key", filterLayer);
      params.set("limit", "500");
      const r = await api.get<{ events: MapEvent[] }>(`/map/events?${params.toString()}`);
      setEvents(r.events);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load");
    } finally {
      setBusy(false);
    }
  }
  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [filterType, filterLayer]);

  async function clearAll() {
    if (!confirm("Delete ALL map events? This cannot be undone.")) return;
    try {
      await api.del(`/map/events/clear`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "clear failed");
    }
  }
  async function del(id: number) {
    if (!confirm("Delete this event?")) return;
    try {
      await api.del(`/map/events/${id}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "delete failed");
    }
  }
  async function postEvent() {
    setPosting(true);
    setError(null);
    try {
      await api.post(`/map/events`, {
        event_type: String(draft.event_type),
        lat: draft.lat === "" || draft.lat == null ? null : Number(draft.lat),
        lon: draft.lon === "" || draft.lon == null ? null : Number(draft.lon),
        zoom: draft.zoom === "" || draft.zoom == null ? null : Number(draft.zoom),
        layer_key: draft.layer_key || null,
        feature_key: draft.feature_key || null,
        data: draft.data || null,
      });
      setPostOpen(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "post failed");
    } finally {
      setPosting(false);
    }
  }

  return (
    <div className="flex h-full flex-col gap-3 overflow-hidden p-4">
      <div className="flex flex-wrap items-center gap-2">
        <Input
          placeholder="filter event type…"
          className="h-8 max-w-[180px] text-xs"
          value={filterType}
          onChange={(e) => setFilterType(e.target.value)}
        />
        <Input
          placeholder="filter layer_key…"
          className="h-8 max-w-[180px] text-xs"
          value={filterLayer}
          onChange={(e) => setFilterLayer(e.target.value)}
        />
        <Button variant="ghost" className="h-7 px-2 text-xs" onClick={() => void load()}>
          Refresh
        </Button>
        {canWriteMaps && (
          <>
            <Button className="h-7 px-2 text-xs" onClick={() => setPostOpen(true)}>
              + Event
            </Button>
            <Button variant="danger" className="h-7 px-2 text-xs" onClick={clearAll}>
              Clear all
            </Button>
          </>
        )}
      </div>
      {error && <Alert tone="error">{error}</Alert>}
      <div className="flex-1 overflow-y-auto rounded border border-[var(--app-border)]">
        {busy ? (
          <div className="grid place-items-center p-6">
            <Spinner />
          </div>
        ) : !events || events.length === 0 ? (
          <EmptyState message="No events." />
        ) : (
          <table className="w-full text-left text-xs">
            <thead className="sticky top-0 bg-[var(--app-card-2)] text-[var(--app-fg-muted)]">
              <tr>
                <th className="px-2 py-1.5">#</th>
                <th className="px-2 py-1.5">Type</th>
                <th className="px-2 py-1.5">Layer / Feature</th>
                <th className="px-2 py-1.5">Lat/Lon</th>
                <th className="px-2 py-1.5">Zoom</th>
                <th className="px-2 py-1.5">When</th>
                <th className="px-2 py-1.5"></th>
              </tr>
            </thead>
            <tbody>
              {events.map((e) => (
                <tr key={e.id} className="border-t border-[var(--app-border)]">
                  <td className="px-2 py-1 font-mono">{e.id}</td>
                  <td className="px-2 py-1">
                    <Badge>{e.event_type}</Badge>
                  </td>
                  <td className="px-2 py-1 font-mono">
                    {e.layer_key ?? "—"}
                    {e.feature_key ? ` / ${e.feature_key}` : ""}
                  </td>
                  <td className="px-2 py-1 font-mono">
                    {e.lat != null && e.lon != null ? `${e.lat.toFixed(4)}, ${e.lon.toFixed(4)}` : "—"}
                  </td>
                  <td className="px-2 py-1">{e.zoom ?? "—"}</td>
                  <td className="px-2 py-1 text-[var(--app-fg-muted)]">
                    {new Date(e.created_at).toLocaleString()}
                  </td>
                  <td className="px-2 py-1 text-right">
                    {canWriteMaps && (
                      <button
                        className="rounded px-1 text-red-500 hover:bg-red-500/10"
                        onClick={() => void del(e.id)}
                      >
                        ×
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <SidePanel open={postOpen} onClose={() => setPostOpen(false)} title="New event">
        <PanelSection label="Event">
          <Field label="Type">
            <select
              className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm"
              value={(draft.event_type as string) ?? "click"}
              onChange={(e) => setDraft((p) => ({ ...p, event_type: e.target.value }))}
            >
              {["click", "hover", "zoom", "pan", "layer_toggle", "search", "error", "custom"].map((t) => (
                <option key={t} value={t}>
                  {t}
                </option>
              ))}
            </select>
          </Field>
          <div className="grid grid-cols-2 gap-2">
            <Field label="Lat">
              <Input
                type="number"
                step="any"
                value={(draft.lat as number | null) ?? ""}
                onChange={(e) => setDraft((p) => ({ ...p, lat: e.target.value }))}
              />
            </Field>
            <Field label="Lon">
              <Input
                type="number"
                step="any"
                value={(draft.lon as number | null) ?? ""}
                onChange={(e) => setDraft((p) => ({ ...p, lon: e.target.value }))}
              />
            </Field>
          </div>
          <Field label="Zoom">
            <Input
              type="number"
              value={(draft.zoom as number | null) ?? ""}
              onChange={(e) => setDraft((p) => ({ ...p, zoom: e.target.value }))}
            />
          </Field>
          <Field label="Layer key">
            <Input
              value={(draft.layer_key as string) ?? ""}
              onChange={(e) => setDraft((p) => ({ ...p, layer_key: e.target.value }))}
            />
          </Field>
          <Field label="Feature key">
            <Input
              value={(draft.feature_key as string) ?? ""}
              onChange={(e) => setDraft((p) => ({ ...p, feature_key: e.target.value }))}
            />
          </Field>
          <Field label="Data (JSON)">
            <Textarea
              rows={4}
              value={(draft.data as string) ?? ""}
              onChange={(e) => setDraft((p) => ({ ...p, data: e.target.value }))}
            />
          </Field>
        </PanelSection>
        <div className="flex justify-end gap-2 pt-4">
          <Button variant="ghost" onClick={() => setPostOpen(false)}>
            Cancel
          </Button>
          <Button onClick={postEvent} loading={posting}>
            Post
          </Button>
        </div>
      </SidePanel>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Form panels (source/layer/style/feature)
// ---------------------------------------------------------------------------

function SourceForm({
  draft,
  setDraft,
  canWrite,
  loadStoredFiles,
  showFilePicker,
  setShowFilePicker,
  storedFiles,
}: {
  draft: Record<string, unknown>;
  setDraft: (fn: (prev: Record<string, unknown>) => Record<string, unknown>) => void;
  canWrite: boolean;
  loadStoredFiles: () => void;
  showFilePicker: boolean;
  setShowFilePicker: (v: boolean) => void;
  storedFiles: StoredFile[] | null;
}) {
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<
    Array<{ id: string; name: string; url?: string; meta?: Record<string, unknown> }> | null
  >(null);

  async function doSearch(q: string) {
    setSearchQuery(q);
    if (!q.trim()) {
      setSearchResults(null);
      return;
    }
    try {
      const res = await api.get<
        Array<{ id: string; name: string; url?: string; meta?: Record<string, unknown> }>
      >(`/search?q=${encodeURIComponent(q.trim())}&limit=10`);
      setSearchResults(res.filter((r) => r.id.startsWith("file:")));
    } catch {
      setSearchResults([]);
    }
  }

  return (
    <PanelSection label="Details">
      <Field label="Name">
        <Input
          value={(draft.name as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, name: e.target.value }))}
        />
      </Field>
      <Field label="Type">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
          value={(draft.source_type as string) ?? "vector"}
          onChange={(e) => setDraft((p) => ({ ...p, source_type: e.target.value }))}
        >
          <option value="vector">Vector (MVT/GeoJSON)</option>
          <option value="raster">Raster (tiles)</option>
          <option value="geojson">GeoJSON (inline)</option>
          <option value="xyz">XYZ tile URL</option>
          <option value="wms">WMS</option>
          <option value="custom">Custom</option>
        </select>
      </Field>
      <Field label="URL">
        <div className="flex gap-2">
          <Input
            className="flex-1"
            value={(draft.url as string) ?? ""}
            onChange={(e) => setDraft((p) => ({ ...p, url: e.target.value }))}
            placeholder="Optional — tile URL or storage path"
          />
          {canWrite && (
            <Button
              variant="ghost"
              onClick={() => {
                setShowFilePicker(!showFilePicker);
                if (!showFilePicker) loadStoredFiles();
              }}
              className="shrink-0"
            >
              Pick file
            </Button>
          )}
        </div>
        {showFilePicker && (
          <div className="mt-2 rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] p-2">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => doSearch(e.target.value)}
              placeholder="Search GeoJSON files…"
              className="w-full rounded border border-[var(--app-border)] bg-[var(--app-card-2)] px-2 py-1 text-sm text-[var(--app-fg)] mb-2"
            />
            {searchResults !== null ? (
              searchResults.length === 0 ? (
                <p className="text-xs text-[var(--app-fg-muted)]">No files match.</p>
              ) : (
                <div className="max-h-40 overflow-y-auto space-y-1">
                  {searchResults.map((r) => (
                    <button
                      key={r.id}
                      className="block w-full rounded px-2 py-1 text-left text-sm hover:bg-[var(--app-card-2)]"
                      onClick={() => {
                        setDraft((p) => ({ ...p, url: r.url, source_type: "geojson" }));
                        setShowFilePicker(false);
                        setSearchQuery("");
                        setSearchResults(null);
                      }}
                    >
                      <span className="font-medium">{r.name}</span>
                      {r.meta && (
                        <span className="ml-2 text-[var(--app-fg-muted)]">
                          {formatBytes((r.meta.size_bytes as number) ?? 0)}
                        </span>
                      )}
                    </button>
                  ))}
                </div>
              )
            ) : storedFiles === null ? (
              <Spinner />
            ) : storedFiles.length === 0 ? (
              <p className="text-xs text-[var(--app-fg-muted)]">No GeoJSON files in storage.</p>
            ) : (
              <div className="max-h-40 overflow-y-auto space-y-1">
                {storedFiles.map((f) => (
                  <button
                    key={f.id}
                    className="block w-full rounded px-2 py-1 text-left text-sm hover:bg-[var(--app-card-2)]"
                    onClick={() => {
                      setDraft((p) => ({
                        ...p,
                        url: `/api/v1/storage/${f.user_id}/${f.collection}/${encodeURIComponent(f.filename)}`,
                        source_type: "geojson",
                      }));
                      setShowFilePicker(false);
                    }}
                  >
                    <span className="font-medium">{f.filename}</span>
                    <span className="ml-2 text-[var(--app-fg-muted)]">{formatBytes(f.size_bytes)}</span>
                  </button>
                ))}
              </div>
            )}
            <button
              className="mt-1 text-xs text-[var(--app-fg-muted)] hover:underline"
              onClick={() => {
                setShowFilePicker(false);
                setSearchQuery("");
                setSearchResults(null);
              }}
            >
              Close
            </button>
          </div>
        )}
      </Field>
      <Field label="Version">
        <Input
          value={(draft.version as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, version: e.target.value }))}
        />
      </Field>
      <Field label="Description">
        <Textarea
          rows={3}
          value={(draft.description as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, description: e.target.value }))}
        />
      </Field>
    </PanelSection>
  );
}

function LayerForm({
  draft,
  setDraft,
  sources,
}: {
  draft: Record<string, unknown>;
  setDraft: (fn: (prev: Record<string, unknown>) => Record<string, unknown>) => void;
  sources: MapSource[];
}) {
  return (
    <PanelSection label="Details">
      <Field label="Name">
        <Input
          value={(draft.name as string) ?? ""}
          onChange={(e) => {
            const name = e.target.value;
            setDraft((p) => ({
              ...p,
              name,
              layer_key:
                p.layer_key === "" || p.layer_key === slugify(p.name as string) ? slugify(name) : p.layer_key,
            }));
          }}
        />
      </Field>
      <Field label="Layer key">
        <Input
          value={(draft.layer_key as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, layer_key: e.target.value }))}
          placeholder="auto-generated from name"
        />
      </Field>
      <Field label="Type">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
          value={(draft.layer_type as string) ?? "vector"}
          onChange={(e) => setDraft((p) => ({ ...p, layer_type: e.target.value }))}
        >
          <option value="vector">Vector</option>
          <option value="raster">Raster</option>
          <option value="geojson">GeoJSON</option>
          <option value="custom">Custom</option>
        </select>
      </Field>
      <Field label="Source">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
          value={(draft.source_id as number) ?? 0}
          onChange={(e) => setDraft((p) => ({ ...p, source_id: Number(e.target.value) }))}
        >
          {sources.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Min zoom">
          <Input
            type="number"
            value={(draft.min_zoom as number) ?? 0}
            onChange={(e) => setDraft((p) => ({ ...p, min_zoom: Number(e.target.value) }))}
          />
        </Field>
        <Field label="Max zoom">
          <Input
            type="number"
            value={(draft.max_zoom as number) ?? 22}
            onChange={(e) => setDraft((p) => ({ ...p, max_zoom: Number(e.target.value) }))}
          />
        </Field>
      </div>
      <Field label="Z-index">
        <Input
          type="number"
          value={(draft.z_index as number) ?? 0}
          onChange={(e) => setDraft((p) => ({ ...p, z_index: Number(e.target.value) }))}
        />
      </Field>
      <Field label="Description">
        <Textarea
          rows={3}
          value={(draft.description as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, description: e.target.value }))}
        />
      </Field>
    </PanelSection>
  );
}

function StyleForm({
  draft,
  setDraft,
  layers,
}: {
  draft: Record<string, unknown>;
  setDraft: (fn: (prev: Record<string, unknown>) => Record<string, unknown>) => void;
  layers: MapLayer[];
}) {
  return (
    <PanelSection label="Details">
      <Field label="Name">
        <Input
          value={(draft.name as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, name: e.target.value }))}
        />
      </Field>
      <Field label="Type">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
          value={(draft.style_type as string) ?? "vector"}
          onChange={(e) => setDraft((p) => ({ ...p, style_type: e.target.value }))}
        >
          <option value="vector">Vector style</option>
          <option value="raster">Raster style</option>
          <option value="json">JSON definition</option>
        </select>
      </Field>
      <Field label="Layer">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
          value={(draft.layer_id as number) ?? ""}
          onChange={(e) =>
            setDraft((p) => ({ ...p, layer_id: e.target.value ? Number(e.target.value) : null }))
          }
        >
          <option value="">None</option>
          {layers.map((l) => (
            <option key={l.id} value={l.id}>
              {l.name}
            </option>
          ))}
        </select>
      </Field>
      <Field label="Definition">
        <StyleDefinitionEditor
          value={(draft.definition as string) ?? "{}"}
          onChange={(json) => setDraft((p) => ({ ...p, definition: json }))}
        />
      </Field>
    </PanelSection>
  );
}

function FeatureForm({
  draft,
  setDraft,
}: {
  draft: Record<string, unknown>;
  setDraft: (fn: (prev: Record<string, unknown>) => Record<string, unknown>) => void;
}) {
  return (
    <PanelSection label="Details">
      <Field label="Key">
        <Input
          value={(draft.feature_key as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, feature_key: e.target.value }))}
        />
      </Field>
      <Field label="Type">
        <Input
          value={(draft.feature_type as string) ?? "feature"}
          onChange={(e) => setDraft((p) => ({ ...p, feature_type: e.target.value }))}
        />
      </Field>
      <Field label="Geometry type">
        <select
          className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)]"
          value={(draft.geometry_type as string) ?? "Point"}
          onChange={(e) => setDraft((p) => ({ ...p, geometry_type: e.target.value }))}
        >
          {["Point", "LineString", "Polygon", "MultiPoint", "MultiLineString", "MultiPolygon"].map((t) => (
            <option key={t} value={t}>
              {t}
            </option>
          ))}
        </select>
      </Field>
      <Field label="Geometry (GeoJSON)">
        <Textarea
          rows={4}
          value={(draft.geometry as string) ?? ""}
          onChange={(e) => setDraft((p) => ({ ...p, geometry: e.target.value }))}
          placeholder='{"type":"Point","coordinates":[0,0]}'
          className="font-mono text-xs"
        />
      </Field>
      <Field label="Properties (JSON)">
        <Textarea
          rows={3}
          value={(draft.properties as string) ?? "{}"}
          onChange={(e) => setDraft((p) => ({ ...p, properties: e.target.value }))}
          className="font-mono text-xs"
        />
      </Field>
    </PanelSection>
  );
}

// ---------------------------------------------------------------------------
// Generic helpers (re-used)
// ---------------------------------------------------------------------------

function useList<T>(url: string, key: string) {
  const [data, setData] = useState<T[] | null>(null);
  useEffect(() => {
    api
      .get<Record<string, T[]>>(url)
      .then((r) => setData((r as Record<string, T[]>)[key] ?? []))
      .catch(() => setData([]));
  }, [url, key]);
  return { data };
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

// ---------------------------------------------------------------------------
// Style definition editor (kept from original)
// ---------------------------------------------------------------------------

interface StyleDef {
  fill?: string;
  fillOpacity?: number;
  stroke?: string;
  color?: string;
  strokeWidth?: number;
  strokeOpacity?: number;
  dashArray?: string;
  radius?: number;
  [key: string]: unknown;
}

const FILL_PROPS = [
  { key: "fill", label: "Fill color", type: "color" as const },
  { key: "fillColor", label: "Fill color (alt)", type: "color" as const },
  { key: "fillOpacity", label: "Fill opacity", type: "number" as const, min: 0, max: 1, step: 0.05 },
];

const STROKE_PROPS = [
  { key: "stroke", label: "Stroke color", type: "color" as const },
  { key: "color", label: "Stroke color (alt)", type: "color" as const },
  { key: "strokeWidth", label: "Stroke width", type: "number" as const, min: 0, max: 20, step: 0.5 },
  { key: "strokeOpacity", label: "Stroke opacity", type: "number" as const, min: 0, max: 1, step: 0.05 },
  { key: "dashArray", label: "Dash array", type: "text" as const, placeholder: "e.g. 8,4" },
];

const MARKER_PROPS = [
  { key: "radius", label: "Radius", type: "number" as const, min: 1, max: 50, step: 1 },
];

const EXTRA_PROPS = [
  { key: "weight", label: "Weight", type: "number" as const, min: 0, max: 20, step: 0.5 },
  { key: "opacity", label: "Opacity", type: "number" as const, min: 0, max: 1, step: 0.05 },
  { key: "zIndex", label: "Z-index", type: "number" as const, min: 0, max: 100, step: 1 },
];

function parseDef(json: string): StyleDef {
  try {
    return JSON.parse(json);
  } catch {
    return {};
  }
}

function defToJson(def: StyleDef): string {
  const clean: Record<string, unknown> = {};
  for (const [k, v] of Object.entries(def)) {
    if (v !== undefined && v !== null && v !== "") clean[k] = v;
  }
  return JSON.stringify(clean, null, 2);
}

function ColorInput({
  label,
  value,
  onChange,
}: {
  label: string;
  value?: string;
  onChange: (v: string) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <input
        type="color"
        className="h-8 w-10 cursor-pointer rounded border border-[var(--app-border)] bg-transparent"
        value={value || "#888888"}
        onChange={(e) => onChange(e.target.value)}
      />
      <div className="flex-1">
        <p className="text-xs text-[var(--app-fg-muted)]">{label}</p>
        <Input
          value={value || ""}
          onChange={(e) => onChange(e.target.value)}
          placeholder="#hex or name"
          className="text-xs"
        />
      </div>
    </div>
  );
}

function NumberInput({
  label,
  value,
  onChange,
  min,
  max,
  step,
}: {
  label: string;
  value?: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
}) {
  return (
    <div>
      <p className="text-xs text-[var(--app-fg-muted)]">{label}</p>
      <input
        type="number"
        className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-1.5 text-sm text-[var(--app-fg)]"
        value={value ?? ""}
        min={min}
        max={max}
        step={step}
        onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
      />
    </div>
  );
}

function TextInput({
  label,
  value,
  onChange,
  placeholder,
}: {
  label: string;
  value?: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  return (
    <div>
      <p className="text-xs text-[var(--app-fg-muted)]">{label}</p>
      <Input
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="text-xs"
      />
    </div>
  );
}

function StylePropertyGroup({
  title,
  icon,
  props,
  def,
  onChange,
}: {
  title: string;
  icon: string;
  props: Array<{
    key: string;
    label: string;
    type: "color" | "number" | "text";
    min?: number;
    max?: number;
    step?: number;
    placeholder?: string;
  }>;
  def: StyleDef;
  onChange: (key: string, value: unknown) => void;
}) {
  const [open, setOpen] = useState(true);
  return (
    <div className="rounded-lg border border-[var(--app-border)] bg-[var(--app-card)]">
      <button
        type="button"
        className="flex w-full items-center justify-between px-3 py-2 text-sm font-medium text-[var(--app-fg)] hover:bg-[var(--app-card-2)]"
        onClick={() => setOpen(!open)}
      >
        <span>
          {icon} {title}
        </span>
        <span className="text-[var(--app-fg-muted)]">{open ? "▾" : "▸"}</span>
      </button>
      {open && (
        <div className="space-y-2 border-t border-[var(--app-border)] px-3 py-2">
          {props.map((p) => {
            const val = def[p.key] as string | number | undefined;
            if (p.type === "color")
              return (
                <ColorInput key={p.key} label={p.label} value={val as string} onChange={(v) => onChange(p.key, v)} />
              );
            if (p.type === "number")
              return (
                <NumberInput
                  key={p.key}
                  label={p.label}
                  value={val as number}
                  onChange={(v) => onChange(p.key, v)}
                  min={p.min}
                  max={p.max}
                  step={p.step}
                />
              );
            return (
              <TextInput
                key={p.key}
                label={p.label}
                value={val as string}
                onChange={(v) => onChange(p.key, v)}
                placeholder={p.placeholder}
              />
            );
          })}
        </div>
      )}
    </div>
  );
}

function StyleDefinitionEditor({ value, onChange }: { value: string; onChange: (json: string) => void }) {
  const def = parseDef(value);
  function set(key: string, val: unknown) {
    const next = { ...def, [key]: val };
    if (val === "" || val === undefined) delete next[key];
    onChange(defToJson(next));
  }
  return (
    <div className="space-y-2">
      <p className="text-xs font-medium text-[var(--app-fg-muted)]">Style Properties</p>
      <StylePropertyGroup title="Fill" icon="◼" props={FILL_PROPS} def={def} onChange={set} />
      <StylePropertyGroup title="Stroke" icon="/" props={STROKE_PROPS} def={def} onChange={set} />
      <StylePropertyGroup title="Marker" icon="●" props={MARKER_PROPS} def={def} onChange={set} />
      <StylePropertyGroup title="Advanced" icon="⚙" props={EXTRA_PROPS} def={def} onChange={set} />
      <details className="rounded-lg border border-[var(--app-border)] bg-[var(--app-card)]">
        <summary className="cursor-pointer px-3 py-2 text-xs font-medium text-[var(--app-fg-muted)] hover:text-[var(--app-fg)]">
          Raw JSON
        </summary>
        <div className="border-t border-[var(--app-border)] px-3 py-2">
          <Textarea rows={4} className="font-mono text-xs" value={value} onChange={(e) => onChange(e.target.value)} />
        </div>
      </details>
    </div>
  );
}
