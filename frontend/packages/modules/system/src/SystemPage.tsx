import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import { Alert, Button, Card, Field, Input, PageHeader, Spinner } from "@gateway/ui";

type SystemInfo = {
  name: string;
  logo_url: string;
  version: string;
};

export function SystemPage({ canWriteSystem = false }: { canWriteSystem?: boolean }) {
  const [system, setSystem] = useState<SystemInfo | null>(null);
  const [name, setName] = useState("");
  const [logoUrl, setLogoUrl] = useState("");
  const [version, setVersion] = useState("");
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    let alive = true;
    api
      .get<{ system: SystemInfo }>("/system")
      .then((res) => {
        if (!alive) return;
        setSystem(res.system);
        setName(res.system.name);
        setLogoUrl(res.system.logo_url);
        setVersion(res.system.version);
      })
      .catch((e) => {
        if (alive)
          setLoadError(
            e instanceof Error ? e.message : "failed to load the system settings",
          );
      });
    return () => {
      alive = false;
    };
  }, []);

  const dirty =
    !!system &&
    (name !== system.name ||
      logoUrl !== system.logo_url ||
      version !== system.version);

  function reset() {
    if (!system) return;
    setName(system.name);
    setLogoUrl(system.logo_url);
    setVersion(system.version);
    setSaveError(null);
    setSaved(false);
  }

  async function save() {
    setSaving(true);
    setSaveError(null);
    setSaved(false);
    try {
      const res = await api.patch<{ system: SystemInfo }>("/system", {
        name,
        logo_url: logoUrl,
        version,
      });
      setSystem(res.system);
      setName(res.system.name);
      setLogoUrl(res.system.logo_url);
      setVersion(res.system.version);
      setSaved(true);
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : "saving failed");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="space-y-8">
      <PageHeader
        title="System Settings"
        description="Configure system name, logo, and version."
      />
      {loadError && <Alert>{loadError}</Alert>}
      {!system && !loadError && (
        <div className="grid place-items-center py-10">
          <Spinner />
        </div>
      )}
      {system && (
        <Card className="max-w-2xl p-6">
          <form
            className="space-y-5"
            onSubmit={(e) => {
              e.preventDefault();
              void save();
            }}
          >
            <Field label="Name">
              <Input
                value={name}
                maxLength={120}
                required
                disabled={!canWriteSystem || saving}
                onChange={(e) => setName(e.target.value)}
              />
            </Field>

            <Field label="Logo URL">
              <Input
                type="url"
                value={logoUrl}
                placeholder="https://…"
                disabled={!canWriteSystem || saving}
                onChange={(e) => setLogoUrl(e.target.value)}
              />
              {logoUrl && (
                <img
                  src={logoUrl}
                  alt="logo preview"
                  className="mt-2 h-12 w-12 rounded-lg border border-[var(--app-border)] object-cover"
                  onError={(e) => {
                    e.currentTarget.style.display = "none";
                  }}
                />
              )}
            </Field>

            <Field label="Version">
              <Input
                value={version}
                maxLength={64}
                disabled={!canWriteSystem || saving}
                onChange={(e) => setVersion(e.target.value)}
              />
            </Field>

            {saveError && <Alert>{saveError}</Alert>}
            {saved && <Alert tone="success">Settings saved.</Alert>}

            {canWriteSystem && (
              <div className="flex gap-2">
                <Button type="submit" loading={saving} disabled={!dirty}>
                  Save changes
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  onClick={reset}
                  disabled={!dirty || saving}
                >
                  Cancel
                </Button>
              </div>
            )}
          </form>
        </Card>
      )}
    </div>
  );
}
