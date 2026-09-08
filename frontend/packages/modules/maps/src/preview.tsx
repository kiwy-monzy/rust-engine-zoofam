import { useEffect, useRef, useState } from "react";
import { api } from "@gateway/lib";
import { MapContainer, useMap } from "react-leaflet";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import { Alert, Badge, Button, Spinner } from "@gateway/ui";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface MapLayer {
  id: number;
  source_id: number;
  name: string;
  layer_key: string;
  layer_type: string;
  visible: boolean;
  style_id: number | null;
  feature_count?: number;
}

interface MapStyle {
  id: number;
  layer_id: number | null;
  name: string;
  style_type: string;
  definition: string;
}

// ---------------------------------------------------------------------------
// Basemap definitions
// ---------------------------------------------------------------------------

interface BasemapDef {
  id: string;
  name: string;
  url: string;
  attribution: string;
  maxZoom: number;
}

const BASEMAPS: BasemapDef[] = [
  { id: "osm", name: "OpenStreetMap", url: "https://tile.openstreetmap.org/{z}/{x}/{y}.png", attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a>', maxZoom: 19 },
  { id: "google", name: "Google Streets", url: "https://mt1.google.com/vt/lyrs=m&x={x}&y={y}&z={z}", attribution: '&copy; Google', maxZoom: 20 },
  { id: "satellite", name: "Google Satellite", url: "https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}", attribution: '&copy; Google', maxZoom: 20 },
  { id: "hybrid", name: "Google Hybrid", url: "https://mt1.google.com/vt/lyrs=y&x={x}&y={y}&z={z}", attribution: '&copy; Google', maxZoom: 20 },
  { id: "carto_light", name: "Carto Light", url: "https://a.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png", attribution: '&copy; CartoDB', maxZoom: 19 },
  { id: "carto_dark", name: "Carto Dark", url: "https://a.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png", attribution: '&copy; CartoDB', maxZoom: 19 },
  { id: "esri", name: "ESRI Satellite", url: "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}", attribution: '&copy; ESRI', maxZoom: 18 },
  { id: "topo", name: "OpenTopoMap", url: "https://a.tile.opentopomap.org/{z}/{x}/{y}.png", attribution: '&copy; OpenTopoMap', maxZoom: 17 },
  { id: "none", name: "No Basemap", url: "", attribution: "", maxZoom: 22 },
];

function isSatellite(id: string): boolean {
  return id === "satellite" || id === "hybrid" || id === "esri";
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const LAYER_COLORS = [
  "#e74c3c", "#3498db", "#2ecc71", "#f39c12", "#9b59b6",
  "#1abc9c", "#e67e22", "#34495e", "#e91e63", "#00bcd4",
  "#8bc34a", "#ff5722", "#607d8b", "#795548", "#cddc39",
];

function parseStyleDef(json: string): Record<string, unknown> {
  try { return JSON.parse(json); } catch { return {}; }
}

function getLayerColor(idx: number): string {
  return LAYER_COLORS[idx % LAYER_COLORS.length];
}

// ---------------------------------------------------------------------------
// VectorGrid dynamic import
// ---------------------------------------------------------------------------

let vectorGridModule: { protobuf: (url: string, opts: Record<string, unknown>) => L.Layer } | null = null;

async function loadVectorGrid() {
  if (vectorGridModule) return vectorGridModule;
  try {
    // @ts-expect-error — leaflet.vectorgrid has no types
    await import("leaflet.vectorgrid");
    const vg = (L as unknown as { vectorGrid?: { protobuf: (url: string, opts: Record<string, unknown>) => L.Layer } }).vectorGrid;
    if (vg) { vectorGridModule = vg; return vg; }
  } catch { /* fallback */ }
  return null;
}

// ---------------------------------------------------------------------------
// VectorTileManager — one VT layer per map layer, toggled on/off
// ---------------------------------------------------------------------------

function VectorTileManager({
  layers,
  styles,
  selectedIds,
  satellite,
}: {
  layers: MapLayer[];
  styles: MapStyle[];
  selectedIds: Set<number>;
  satellite: boolean;
}) {
  const map = useMap();
  const vtLayersRef = useRef<Map<number, L.Layer>>(new Map());

  useEffect(() => {
    let cancelled = false;

    async function sync() {
      const vg = await loadVectorGrid();
      if (cancelled || !vg) return;

      for (const layer of layers) {
        const shouldShow = selectedIds.has(layer.id);
        const existing = vtLayersRef.current.get(layer.id);

        if (shouldShow && !existing) {
          const style = styles.find((s) => s.layer_id === layer.id);
          const def = style ? parseStyleDef(style.definition) : {};
          const idx = layers.indexOf(layer);
          const fallbackColor = getLayerColor(idx);

          const vtLayer = vg.protobuf(`/api/v1/tiles/${layer.layer_key}/{z}/{x}/{y}`, {
            maxZoom: 22,
            vectorTileLayerStyles: {
              [layer.layer_key]: (props: Record<string, unknown>, _zoom: number) => {
                const geomType = (props.geomType as string) || (props.geometry_type as string) || "";
                const g = geomType.toLowerCase();
                const weightMult = satellite ? 1.6 : 1;
                const sw = (def.strokeWidth as number) || 2;
                const swThin = (def.strokeWidth as number) || 1;

                if (style) {
                  if (g.includes("line")) {
                    return {
                      color: def.stroke || def.color || "#333",
                      weight: sw * weightMult,
                      opacity: def.strokeOpacity ?? 1,
                      dashArray: def.dashArray as string | undefined,
                    };
                  }
                  if (g.includes("polygon")) {
                    return {
                      color: def.stroke || def.color || "#333",
                      fillColor: def.fill || def.fillColor || "#88f",
                      fillOpacity: def.fillOpacity ?? 0.3,
                      weight: swThin * weightMult,
                      opacity: def.strokeOpacity ?? 1,
                    };
                  }
                  return {
                    fillColor: def.fillColor || def.fill || "#f00",
                    color: def.stroke || "#fff",
                    weight: sw * weightMult,
                    radius: (def.radius as number) || 8,
                    fillOpacity: def.fillOpacity ?? 0.9,
                  };
                }
                // Default styling by layer index
                if (g.includes("line")) {
                  const w = layer.layer_key.includes("trunk") ? 3 : 1.5;
                  return {
                    color: fallbackColor,
                    weight: w * weightMult,
                    opacity: 0.9,
                  };
                }
                if (g.includes("polygon")) {
                  return {
                    color: fallbackColor,
                    fillColor: fallbackColor,
                    fillOpacity: 0.25,
                    weight: weightMult,
                  };
                }
                return {
                  fillColor: fallbackColor,
                  color: "#fff",
                  weight: 2 * weightMult,
                  radius: 6,
                  fillOpacity: 0.9,
                };
              },
            },
            interactive: true,
            getFeatureId: (f: { properties?: Record<string, unknown> }) => f.properties?.id,
          }).addTo(map);

          // Click popup with feature properties + label
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          vtLayer.on("click", (e: any) => {
            const layerEvent = e.layer as { properties?: Record<string, unknown> } | undefined;
            const props = layerEvent?.properties;
            if (!props) return;

            const label = (props.label as string) || "";
            const featureKey = (props.feature_key as string) || "";
            const lines = Object.entries(props)
              .filter(([k]) => !["id", "geomType", "geometry_type", "label", "feature_key", "layer_id", "feature_type"].includes(k))
              .map(([k, v]) => `<tr><td class="pr-3 text-[var(--app-fg-muted)]">${k}</td><td class="font-medium">${v}</td></tr>`)
              .join("");

            const html = `
              <div class="min-w-[180px]">
                ${label ? `<div class="mb-1 text-base font-bold">${label}</div>` : ""}
                <div class="mb-2 text-xs text-[var(--app-fg-muted)]">${layer.name} / ${featureKey}</div>
                ${lines ? `<table class="text-xs">${lines}</table>` : ""}
              </div>
            `;
            const center = map.getCenter();
            L.popup({ maxWidth: 320, className: "gw-popup" })
              .setLatLng(center)
              .setContent(html)
              .openOn(map);
          });

          vtLayersRef.current.set(layer.id, vtLayer);
        } else if (!shouldShow && existing) {
          map.removeLayer(existing);
          vtLayersRef.current.delete(layer.id);
        }
      }

      for (const [id, vt] of vtLayersRef.current) {
        if (!layers.find((l) => l.id === id)) {
          map.removeLayer(vt);
          vtLayersRef.current.delete(id);
        }
      }

      // Bring all vector tile layers to front so they render above basemap.
      for (const [, vt] of vtLayersRef.current) {
        (vt as any).bringToFront?.();
      }
    }

    void sync();
    return () => { cancelled = true; };
  }, [map, layers, styles, selectedIds, satellite]);

  useEffect(() => {
    return () => {
      for (const [, vt] of vtLayersRef.current) map.removeLayer(vt);
      vtLayersRef.current.clear();
    };
  }, [map]);

  return null;
}

// ---------------------------------------------------------------------------
// Dynamic basemap layer — swaps when basemap changes
// ---------------------------------------------------------------------------

function BasemapLayer({ basemap }: { basemap: BasemapDef }) {
  const map = useMap();
  const layerRef = useRef<L.TileLayer | null>(null);

  useEffect(() => {
    if (layerRef.current) {
      map.removeLayer(layerRef.current);
      layerRef.current = null;
    }
    if (basemap.url) {
      const layer = L.tileLayer(basemap.url, {
        maxZoom: basemap.maxZoom,
        attribution: basemap.attribution,
      }).addTo(map);
      layer.bringToBack();
      layerRef.current = layer;
    }
  }, [map, basemap]);

  useEffect(() => {
    return () => {
      if (layerRef.current) {
        map.removeLayer(layerRef.current);
        layerRef.current = null;
      }
    };
  }, [map]);

  return null;
}

// ---------------------------------------------------------------------------
// FleetLayerManager — three optional MVT layers (bolt / marine / flights)
// sourced from /api/v1/fleet/tiles/{source}/{z}/{x}/{y}.
// ---------------------------------------------------------------------------

const FLEET_SOURCES = [
  { id: "bolt", label: "Bolt", color: "#3b82f6" },
  { id: "marine", label: "Marine", color: "#06b6d4" },
  { id: "flights", label: "Flights", color: "#ef4444" },
] as const;

type FleetSourceId = (typeof FLEET_SOURCES)[number]["id"];

function FleetLayerManager({
  enabled,
  iconForSource,
}: {
  enabled: Set<FleetSourceId>;
  iconForSource: Record<FleetSourceId, (props: Record<string, unknown>, zoom: number) => Record<string, unknown>>;
}) {
  const map = useMap();
  const layersRef = useRef<Map<string, L.Layer>>(new Map());

  useEffect(() => {
    let cancelled = false;

    async function sync() {
      const vg = await loadVectorGrid();
      if (cancelled || !vg) return;

      for (const src of FLEET_SOURCES) {
        const key = src.id;
        const shouldShow = enabled.has(key);
        const existing = layersRef.current.get(key);

        if (shouldShow && !existing) {
          const vtLayer = vg.protobuf(`/api/v1/fleet/tiles/${key}/{z}/{x}/{y}`, {
            maxZoom: 22,
            vectorTileLayerStyles: {
              [key]: (props: Record<string, unknown>, _zoom: number) => {
                const vt = iconForSource[key]?.(props, _zoom);
                return vt ?? { radius: 4, color: src.color, fillColor: src.color, weight: 1, fillOpacity: 0.8 };
              },
            },
            interactive: true,
            getFeatureId: (f: { properties?: Record<string, unknown> }) => f.properties?.id as string | undefined,
          }).addTo(map);

          vtLayer.on("click", (e: { layer?: { properties?: Record<string, unknown> } }) => {
            const props = e.layer?.properties;
            if (!props) return;
            const label = (props.label as string) || (props.name as string) || key;
            const html = `<div class="min-w-[180px]"><div class="mb-1 text-sm font-bold">${label}</div><div class="text-[10px] text-[var(--app-fg-muted)]">${key}</div></div>`;
            L.popup({ maxWidth: 320, className: "gw-popup" })
              .setLatLng(map.getCenter())
              .setContent(html)
              .openOn(map);
          });

          layersRef.current.set(key, vtLayer);
        } else if (!shouldShow && existing) {
          map.removeLayer(existing);
          layersRef.current.delete(key);
        }
      }
    }

    void sync();
    return () => {
      cancelled = true;
    };
  }, [map, enabled, iconForSource]);

  useEffect(() => {
    return () => {
      for (const [, vt] of layersRef.current) map.removeLayer(vt);
      layersRef.current.clear();
    };
  }, [map]);

  return null;
}

// ---------------------------------------------------------------------------
// Fit-bounds controller
// ---------------------------------------------------------------------------

function FitBoundsController({ layers, selectedIds }: { layers: MapLayer[]; selectedIds: Set<number> }) {
  const map = useMap();
  const fitted = useRef(false);

  useEffect(() => {
    if (fitted.current) return;
    const visible = layers.filter((l) => selectedIds.has(l.id));
    if (visible.length === 0) return;

    const layer = visible[0];
    fetch(`/api/v1/tiles/${layer.layer_key}/0/0/0.json`)
      .then((r) => r.json())
      .then((fc: GeoJSON.FeatureCollection) => {
        if (!fc.features?.length) return;
        const coords: [number, number][] = [];
        for (const f of fc.features) {
          const g = f.geometry;
          if (g.type === "Point") coords.push(g.coordinates as [number, number]);
          else if (g.type === "MultiPoint" || g.type === "LineString") {
            for (const c of g.coordinates) coords.push(c as [number, number]);
          } else if (g.type === "MultiLineString" || g.type === "Polygon") {
            for (const ring of g.coordinates) for (const c of ring) coords.push(c as [number, number]);
          } else if (g.type === "MultiPolygon") {
            for (const poly of g.coordinates) for (const ring of poly) for (const c of ring) coords.push(c as [number, number]);
          }
        }
        if (coords.length > 0) {
          const lats = coords.map((c) => c[1]);
          const lngs = coords.map((c) => c[0]);
          map.fitBounds(
            L.latLngBounds([Math.min(...lats), Math.min(...lngs)], [Math.max(...lats), Math.max(...lngs)]),
            { padding: [20, 20] },
          );
          fitted.current = true;
        }
      })
      .catch(() => {});
  }, [map, layers, selectedIds]);

  return null;
}

// ---------------------------------------------------------------------------
// Main preview component
// ---------------------------------------------------------------------------

export function MapPreviewTab({ canWrite: _canWrite }: { canWrite: boolean }) {
  const [layers, setLayers] = useState<MapLayer[]>([]);
  const [styles, setStyles] = useState<MapStyle[]>([]);
  const [fleetEnabled, setFleetEnabled] = useState<Set<FleetSourceId>>(new Set());
  const [fleetCounts, setFleetCounts] = useState<{ bolt: number; marine: number; flights: number; cached: boolean }>({
    bolt: 0,
    marine: 0,
    flights: 0,
    cached: false,
  });
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(true);
  const [selectedLayerIds, setSelectedLayerIds] = useState<Set<number>>(new Set());
  const [basemapId, setBasemapId] = useState("osm");

  const basemap = BASEMAPS.find((b) => b.id === basemapId) || BASEMAPS[0];
  const satellite = isSatellite(basemapId);

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const layRes = await api.get<{ layers: MapLayer[] }>("/map/layers");
      setLayers(layRes.layers);
      try {
        const styRes = await api.get<{ styles: MapStyle[] }>("/map/styles");
        setStyles(styRes.styles);
      } catch { /* no styles */ }
      const visibleLayers = layRes.layers.filter((l) => l.visible);
      setSelectedLayerIds(new Set(visibleLayers.map((l) => l.id)));
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load");
    } finally {
      setBusy(false);
    }
  }

  async function loadFleet() {
    try {
      const s = await api.get<{
        counts: { bolt: number; marine: number; flights: number };
        cached: boolean;
      }>("/fleet/summary");
      setFleetCounts({ ...s.counts, cached: s.cached });
    } catch {
      /* fleet offline */
    }
  }

  useEffect(() => { void load(); void loadFleet(); }, []);

  useEffect(() => {
    const t = setInterval(() => void loadFleet(), 15_000);
    return () => clearInterval(t);
  }, []);

  function toggleLayer(id: number) {
    setSelectedLayerIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }

  function toggleFleet(id: FleetSourceId) {
    setFleetEnabled((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }

  function toggleAll(on: boolean) {
    if (on) setSelectedLayerIds(new Set(layers.map((l) => l.id)));
    else setSelectedLayerIds(new Set());
  }

  return (
    <div className="space-y-4">
      {error && <Alert tone="error">{error}</Alert>}

      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-[var(--app-fg)]">Layer Preview</h3>
        <div className="flex gap-2">
          <select
            className="rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-2 py-1 text-xs text-[var(--app-fg)]"
            value={basemapId}
            onChange={(e) => setBasemapId(e.target.value)}
          >
            {BASEMAPS.map((b) => (
              <option key={b.id} value={b.id}>{b.name}</option>
            ))}
          </select>
          <Button variant="ghost" onClick={() => toggleAll(true)}>All on</Button>
          <Button variant="ghost" onClick={() => toggleAll(false)}>All off</Button>
          <Button variant="ghost" onClick={() => void load()}>Refresh</Button>
        </div>
      </div>

      {busy && <Spinner />}

      {!busy && layers.length === 0 && (
        <Alert tone="info">No layers yet. Create layers and import GeoJSON features first.</Alert>
      )}

      <div className="flex gap-4" style={{ height: "70vh" }}>
        {/* Layer sidebar */}
        <div className="w-64 shrink-0 space-y-1 overflow-y-auto rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] p-3">
          <p className="mb-2 text-xs font-medium uppercase text-[var(--app-fg-muted)]">
            Live fleet
          </p>
          {FLEET_SOURCES.map((src) => {
            const on = fleetEnabled.has(src.id);
            const count = fleetCounts[src.id];
            return (
              <label
                key={src.id}
                className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 hover:bg-[var(--app-card-2)]"
              >
                <input
                  type="checkbox"
                  checked={on}
                  onChange={() => toggleFleet(src.id)}
                  className="accent-current"
                />
                <span
                  className="inline-block h-3 w-3 rounded-full"
                  style={{ backgroundColor: src.color }}
                />
                <span className="flex-1 truncate text-sm">{src.label}</span>
                <Badge>
                  {count} {fleetCounts.cached ? "" : "—"}
                </Badge>
              </label>
            );
          })}

          <hr className="my-3 border-[var(--app-border)]" />

          <p className="mb-2 text-xs font-medium uppercase text-[var(--app-fg-muted)]">
            Map layers
          </p>
          {layers.map((layer, idx) => (
            <label
              key={layer.id}
              className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 hover:bg-[var(--app-card-2)]"
            >
              <input
                type="checkbox"
                checked={selectedLayerIds.has(layer.id)}
                onChange={() => toggleLayer(layer.id)}
                className="accent-current"
              />
              <span
                className="inline-block h-3 w-3 rounded-full"
                style={{ backgroundColor: getLayerColor(idx) }}
              />
              <span className="flex-1 truncate text-sm">{layer.name}</span>
              <Badge>{layer.layer_key}</Badge>
            </label>
          ))}
        </div>

        {/* Map */}
        <div className="flex-1 overflow-hidden rounded-lg border border-[var(--app-border)]">
          {busy ? (
            <div className="flex h-full items-center justify-center"><Spinner /></div>
          ) : (
            <MapContainer
              center={[-6.37, 34.89]}
              zoom={6}
              style={{ height: "100%", width: "100%" }}
              className="bg-[var(--app-bg)]"
            >
              <BasemapLayer basemap={basemap} />
              <FitBoundsController layers={layers} selectedIds={selectedLayerIds} />
              <VectorTileManager
                layers={layers}
                styles={styles}
                selectedIds={selectedLayerIds}
                satellite={satellite}
              />
              <FleetLayerManager
                enabled={fleetEnabled}
                iconForSource={{
                  bolt: (_props, _zoom) => ({ radius: 5, color: "#1d4ed8", fillColor: "#3b82f6", weight: 1, fillOpacity: 0.9 }),
                  marine: (_props, _zoom) => ({ radius: 4, color: "#0e7490", fillColor: "#06b6d4", weight: 1, fillOpacity: 0.9 }),
                  flights: (_props, _zoom) => ({ radius: 6, color: "#b91c1c", fillColor: "#ef4444", weight: 1, fillOpacity: 0.9 }),
                }}
              />
            </MapContainer>
          )}
        </div>
      </div>

      {/* Stats */}
      {!busy && (
        <div className="flex flex-wrap gap-4 text-sm text-[var(--app-fg-muted)]">
          <span>{layers.length} layer{layers.length !== 1 ? "s" : ""}</span>
          <span>{selectedLayerIds.size} visible</span>
          <span>{layers.reduce((sum, l) => sum + (l.feature_count ?? 0), 0)} total features</span>
          {fleetEnabled.size > 0 && (
            <span>
              Fleet: {fleetEnabled.size} source{fleetEnabled.size === 1 ? "" : "s"} on
            </span>
          )}
        </div>
      )}
    </div>
  );
}
