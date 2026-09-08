import { useEffect, useState } from "react";
import { api } from "@gateway/lib";
import type { Recipient, TicketView } from "@gateway/lib";
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
  Textarea,
  cn,
} from "@gateway/ui";

const CATEGORIES = {
  viewer_to_admin: "Viewer → Admin",
  admin_to_admin: "Admin → Admin",
  viewer_to_viewer: "Viewer → Viewer",
} as const;

type Category = keyof typeof CATEGORIES;

export function SupportPage({
  userId,
  isAdmin = false,
  canWriteSupport = false,
}: {
  userId: string;
  isAdmin?: boolean;
  canWriteSupport?: boolean;
}) {
  const [tickets, setTickets] = useState<TicketView[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);

  const [composeOpen, setComposeOpen] = useState(false);
  const [recipients, setRecipients] = useState<Recipient[] | null>(null);

  const [category, setCategory] = useState<Category>(
    isAdmin ? "admin_to_admin" : "viewer_to_admin",
  );
  const [recipientId, setRecipientId] = useState("");
  const [subject, setSubject] = useState("");
  const [body, setBody] = useState("");
  const [formError, setFormError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);

  const [selected, setSelected] = useState<TicketView | null>(null);

  async function load() {
    try {
      const res = await api.get<{ tickets: TicketView[] }>("/support");
      setTickets(res.tickets);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load the tickets");
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function openCompose() {
    setComposeOpen(true);
    setFormError(null);
    if (isAdmin) return;
    try {
      const res = await api.get<{ users: Recipient[] }>("/support/users");
      setRecipients(res.users);
    } catch (e) {
      setFormError(e instanceof Error ? e.message : "failed to load recipients");
    }
  }

  function closeCompose() {
    setComposeOpen(false);
    setCategory(isAdmin ? "admin_to_admin" : "viewer_to_admin");
    setRecipientId("");
    setSubject("");
    setBody("");
    setFormError(null);
  }

  function canClose(t: TicketView): boolean {
    if (!canWriteSupport || t.status !== "open") return false;
    return (
      t.sender_id === userId ||
      t.recipient_id === userId ||
      (isAdmin && t.recipient_id === null)
    );
  }

  async function closeTicket(id: string) {
    setBusyId(id);
    setError(null);
    try {
      await api.post(`/support/${id}/close`);
      await load();
      setSelected((s) => (s?.id === id ? { ...s, status: "closed" } : s));
    } catch (e) {
      setError(e instanceof Error ? e.message : "closing failed");
    } finally {
      setBusyId(null);
    }
  }

  async function send() {
    const trimmedSubject = subject.trim();
    if (!trimmedSubject) {
      setFormError("the subject must be 1-200 characters");
      return;
    }
    if (!body.trim()) {
      setFormError("the ticket needs a message");
      return;
    }
    if (category === "viewer_to_viewer" && !recipientId) {
      setFormError("viewer_to_viewer tickets need a recipient");
      return;
    }
    setSending(true);
    setFormError(null);
    try {
      await api.post("/support", {
        category,
        recipient_id: category === "viewer_to_viewer" ? recipientId : null,
        subject: trimmedSubject,
        body: body.trim(),
      });
      closeCompose();
      await load();
    } catch (e) {
      setFormError(e instanceof Error ? e.message : "sending failed");
    } finally {
      setSending(false);
    }
  }

  const categories: Category[] = isAdmin
    ? ["admin_to_admin", "viewer_to_viewer"]
    : ["viewer_to_admin", "viewer_to_viewer"];

  return (
    <div className="space-y-8">
      <PageHeader
        title="Support"
        description="Tickets you sent, tickets addressed to you, and role-addressed tickets if you are an admin."
        actions={
          canWriteSupport && (
            <Button onClick={() => void openCompose()}>New ticket</Button>
          )
        }
      />
      {error && <Alert>{error}</Alert>}

      {!tickets ? (
        <div className="grid place-items-center py-10">
          <Spinner />
        </div>
      ) : tickets.length === 0 ? (
        <EmptyState message="No tickets yet." />
      ) : (
        <Table head={["Subject", "Category", "From", "To", "Status", "Updated"]}>
          {tickets.map((t) => (
            <tr
              key={t.id}
              onClick={() => setSelected(t)}
              className={cn(
                "cursor-pointer transition-colors hover:bg-[var(--sb-hover)]",
                selected?.id === t.id && "bg-[var(--accent-soft)]",
              )}
            >
              <Td>
                <span className="font-medium">{t.subject}</span>
              </Td>
              <Td>
                <Badge>{CATEGORIES[t.category as Category] ?? t.category}</Badge>
              </Td>
              <Td className="text-[var(--app-fg-muted)]">{t.sender}</Td>
              <Td className="text-[var(--app-fg-muted)]">
                {t.recipient ?? <span className="text-[var(--app-fg-faint)]">role</span>}
              </Td>
              <Td>
                <Badge tone={t.status === "open" ? "green" : "zinc"}>{t.status}</Badge>
              </Td>
              <Td className="text-[var(--app-fg-muted)]">
                {new Date(t.updated_at + "Z").toLocaleString()}
              </Td>
            </tr>
          ))}
        </Table>
      )}

      <SidePanel
        open={!!selected}
        onClose={() => setSelected(null)}
        title={selected?.subject ?? ""}
        subtitle={
          selected
            ? `${selected.sender} → ${selected.recipient ?? CATEGORIES[selected.category as Category]}`
            : undefined
        }
      >
        {selected && (
          <>
            <PanelSection label="Ticket">
              <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-xs">
                <dt className="text-[var(--app-fg-faint)]">Category</dt>
                <dd>
                  <Badge>{CATEGORIES[selected.category as Category] ?? selected.category}</Badge>
                </dd>
                <dt className="text-[var(--app-fg-faint)]">Status</dt>
                <dd>
                  <Badge tone={selected.status === "open" ? "green" : "zinc"}>
                    {selected.status}
                  </Badge>
                </dd>
                <dt className="text-[var(--app-fg-faint)]">Created</dt>
                <dd className="text-[var(--app-fg-muted)]">
                  {new Date(selected.created_at + "Z").toLocaleString()}
                </dd>
                <dt className="text-[var(--app-fg-faint)]">Updated</dt>
                <dd className="text-[var(--app-fg-muted)]">
                  {new Date(selected.updated_at + "Z").toLocaleString()}
                </dd>
              </dl>
            </PanelSection>

            <PanelSection label="Message">
              <p className="whitespace-pre-wrap text-sm leading-relaxed text-[var(--app-fg)]">
                {selected.body}
              </p>
            </PanelSection>

            {canClose(selected) && (
              <PanelSection label="Actions">
                <Button
                  variant="danger"
                  className="w-full"
                  loading={busyId === selected.id}
                  onClick={() => void closeTicket(selected.id)}
                >
                  Close ticket
                </Button>
              </PanelSection>
            )}
          </>
        )}
      </SidePanel>

      <SidePanel
        open={composeOpen}
        onClose={closeCompose}
        title="New ticket"
        subtitle={CATEGORIES[category]}
      >
        <form
          className="space-y-5"
          onSubmit={(e) => {
            e.preventDefault();
            void send();
          }}
        >
          <PanelSection label="Lane">
            <div className="flex flex-wrap gap-2">
              {categories.map((c) => (
                <button
                  key={c}
                  type="button"
                  onClick={() => {
                    setCategory(c);
                    setFormError(null);
                  }}
                  className={cn(
                    "rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors",
                    category === c
                      ? "border-transparent bg-[var(--accent)] text-white"
                      : "border-[var(--app-border)] bg-[var(--app-card-2)] text-[var(--app-fg-muted)] hover:bg-[var(--sb-hover)]",
                  )}
                >
                  {CATEGORIES[c]}
                </button>
              ))}
            </div>
            {category === "viewer_to_viewer" && (
              <Field label="Recipient">
                {!recipients ? (
                  <Spinner className="h-4 w-4" />
                ) : (
                  <select
                    value={recipientId}
                    onChange={(e) => setRecipientId(e.target.value)}
                    disabled={sending}
                    className="w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] px-3 py-2 text-sm text-[var(--app-fg)] focus:border-[var(--accent)] focus:outline-none"
                  >
                    <option value="">Choose a user…</option>
                    {recipients.map((r) => (
                      <option key={r.id} value={r.id}>
                        {r.name}
                      </option>
                    ))}
                  </select>
                )}
              </Field>
            )}
            {category === "viewer_to_admin" && (
              <p className="text-xs text-[var(--app-fg-muted)]">
                Every admin will see this ticket.
              </p>
            )}
            {category === "admin_to_admin" && (
              <p className="text-xs text-[var(--app-fg-muted)]">
                Visible to admins only.
              </p>
            )}
          </PanelSection>

          <PanelSection label="Message">
            <Field label="Subject">
              <Input
                value={subject}
                maxLength={200}
                disabled={sending}
                onChange={(e) => setSubject(e.target.value)}
              />
            </Field>
            <Field label="Body">
              <Textarea
                rows={8}
                value={body}
                disabled={sending}
                onChange={(e) => setBody(e.target.value)}
              />
            </Field>
          </PanelSection>

          {formError && <Alert>{formError}</Alert>}

          <div className="flex gap-2">
            <Button type="submit" loading={sending}>
              Send ticket
            </Button>
            <Button type="button" variant="ghost" onClick={closeCompose} disabled={sending}>
              Cancel
            </Button>
          </div>
        </form>
      </SidePanel>
    </div>
  );
}
