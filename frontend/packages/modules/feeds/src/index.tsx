/**
 * @gateway/module-feeds — Dashboard FeedsCard module
 *
 * Ported from the Builder.io JSON "Feed Section" (Ko-fi tip feed) supplied by the user:
 * frontend/src blocks builder-mtbc9vrk... — 12 tip entries, avatars, timestamps,
 * optional message bubbles, Give support CTA, and Show more pagination.
 *
 * Exposes two exports:
 *  - FeedsCard  — the card itself, usable as a dashboard widget
 *  - FeedsPage  — a full PageHeader + FeedsCard page suitable for /dashboard or /feeds
 */

import { useState } from "react";
import { Card, PageHeader } from "@gateway/ui";

// ------------------------------------------------------------------ types --

export type FeedItem = {
  id: string;
  name: string;
  avatar: string;
  href?: string;
  timeLabel: string; // e.g. "7h", "2 days ago"
  message?: string | null;
};

// Hard-extracted from the Builder JSON in the user message (blocks[0].children).
// Keeps anon avatar URLs, ko-fi links where present, and message bubbles where rendered.
const FEED_ITEMS: FeedItem[] = [
  {
    id: "1",
    name: "Ali",
    avatar: "https://ko-fi.com/img/anon6.png?v=11",
    href: "https://ko-fi.com/S4Z124LTRM",
    timeLabel: "7h",
    message: null,
  },
  {
    id: "2",
    name: "Supporter",
    avatar: "https://ko-fi.com/img/anon5.png?v=11",
    href: "https://ko-fi.com/R0K525OE98",
    timeLabel: "2 days ago",
    message: null,
  },
  {
    id: "3",
    name: "Supporter",
    avatar: "https://ko-fi.com/img/anon7.png?v=11",
    href: undefined,
    timeLabel: "3 days ago",
    message: null,
  },
  {
    id: "4",
    name: "Bruno Campello",
    avatar: "https://ko-fi.com/img/anon5.png?v=11",
    href: "https://ko-fi.com/O0C625EC09",
    timeLabel: "7 days ago",
    message: "I'm form Brazil. The BRL currency here is garbage.",
  },
  {
    id: "5",
    name: "Supporter",
    avatar: "https://ko-fi.com/img/anon7.png?v=11",
    href: undefined,
    timeLabel: "8 days ago",
    message: null,
  },
  {
    id: "6",
    name: "Supporter",
    avatar: "https://ko-fi.com/img/anon7.png?v=11",
    href: undefined,
    timeLabel: "8 days ago",
    message: null,
  },
  {
    id: "7",
    name: "Damir",
    avatar: "https://ko-fi.com/img/anon6.png?v=11",
    href: "https://ko-fi.com/E3Q0258L8Q",
    timeLabel: "9 days ago",
    message: "Greetings from Novi Sad, Serbia",
  },
  {
    id: "8",
    name: "Manu",
    avatar: "https://ko-fi.com/img/anon2.png?v=11",
    href: undefined,
    timeLabel: "12 days ago",
    message: "Thank you!",
  },
  {
    id: "9",
    name: "Mike",
    avatar: "https://ko-fi.com/img/anon2.png?v=11",
    href: undefined,
    timeLabel: "12 days ago",
    message: "Love this extension for Claude.  I am constantly looking at it to manage my usage.  Great job.",
  },
  {
    id: "10",
    name: "Supporter",
    avatar: "https://ko-fi.com/img/anon7.png?v=11",
    href: undefined,
    timeLabel: "14 days ago",
    message: null,
  },
  {
    id: "11",
    name: "asozzi",
    avatar: "https://ko-fi.com/img/anon4.png?v=11",
    href: undefined,
    timeLabel: "14 days ago",
    message: "Gread Job",
  },
  {
    id: "12",
    name: "GC",
    avatar: "https://storage.ko-fi.com/cdn/useruploads/post/403e083e-1a69-4023-be3d-73dfc67d455c_fc6a1e7d-0039-48d2-93fd-be1b262139a3.png",
    href: "https://ko-fi.com/S4X724U7DI",
    timeLabel: "16 days ago",
    message: "Thanks for all your help!",
  },
];

// --------------------------------------------------------------- helpers --

function openDonationModal() {
  // Original Builder JSON used onclick="openDonationModal()".
  // In the SPA we emit a soft placeholder — wire to real payment flow when available.
  window.dispatchEvent(new CustomEvent("gateway:openDonationModal"));
  // fallback visible feedback if no listener
  setTimeout(() => {
    if (typeof window !== "undefined") {
      // avoid stacking if handler prevented default
      // eslint-disable-next-line no-alert
      const hasListener = false;
      void hasListener;
    }
  }, 0);
}

// --------------------------------------------------------------- FeedsCard --

export function FeedsCard({
  items = FEED_ITEMS,
  pageSize = 4,
  maxWidth = 550,
}: {
  items?: FeedItem[];
  pageSize?: number;
  maxWidth?: number;
}) {
  const [visible, setVisible] = useState(pageSize);
  const [donationToast, setDonationToast] = useState(false);
  const [openMenu, setOpenMenu] = useState<string | null>(null);

  const sliced = items.slice(0, visible);
  const canShowMore = visible < items.length;

  function handleGiveSupport() {
    openDonationModal();
    setDonationToast(true);
    setTimeout(() => setDonationToast(false), 2800);
  }

  function handleShowMore() {
    setVisible((v) => Math.min(v + pageSize, items.length));
  }

  return (
    <div
      style={{ maxWidth, fontFamily: '"DM Sans", Nunito, sans-serif' }}
      className="w-full text-[16px] leading-[22.8571px]"
    >
      {/* Outer card — mirrors builder-mtbc9vrk058ll8tf3s13 responsiveStyles */}
      <div className="rounded-[18px] bg-white p-4 shadow-sm border border-[rgb(212,207,205)] dark:border-[var(--app-border)] dark:bg-[var(--app-card)]">
        {/* Header: Feed + Give support */}
        <div className="mb-2 flex items-center justify-between">
          <span className="font-semibold text-[var(--app-fg)] dark:text-[var(--app-fg)] text-[#111]">Feed</span>
          <button
            type="button"
            onClick={handleGiveSupport}
            className="text-[14px] text-[rgb(129,129,129)] underline decoration-[rgb(129,129,129)] underline-offset-2 transition-colors hover:text-[rgb(90,90,90)] dark:text-[var(--app-fg-muted)]"
          >
            Give support
          </button>
        </div>

        {donationToast && (
          <div className="mb-3 rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800 dark:border-amber-900/40 dark:bg-amber-950/40 dark:text-amber-200">
            Donation flow — hook <code className="font-mono text-xs">gateway:openDonationModal</code> to open your checkout.
          </div>
        )}

        {/* Feed list */}
        <div className="space-y-4">
          {sliced.map((item) => (
            <div key={item.id} className="pb-1">
              {/* Row: avatar + name + tip + time + menu */}
              <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-0">
                  <div className="mr-2">
                    {item.href ? (
                      <a href={item.href} target="_blank" rel="noreferrer" className="inline-block">
                        <img
                          src={item.avatar}
                          alt={item.name}
                          loading="lazy"
                          className="h-[38px] w-[38px] rounded-full object-cover align-middle"
                        />
                      </a>
                    ) : (
                      <img
                        src={item.avatar}
                        alt={item.name}
                        loading="lazy"
                        className="h-[38px] w-[38px] rounded-full object-cover align-middle"
                      />
                    )}
                  </div>
                  <div className="flex flex-col items-start">
                    <div className="flex flex-wrap items-baseline gap-1">
                      {item.href ? (
                        <a
                          href={item.href}
                          target="_blank"
                          rel="noreferrer"
                          className="inline-block max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap align-bottom font-semibold text-[#111] hover:underline dark:text-[var(--app-fg)]"
                        >
                          {item.name}
                        </a>
                      ) : (
                        <span className="inline-block max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap align-bottom font-semibold text-[#111] dark:text-[var(--app-fg)]">
                          {item.name}
                        </span>
                      )}
                      <span className="text-[#111] dark:text-[var(--app-fg)]">gave a tip!</span>
                      <span className="whitespace-nowrap text-[13px] leading-[18.5714px] text-[rgb(132,138,149)]">
                        {" "}
                        · {item.timeLabel}
                      </span>
                    </div>
                  </div>
                </div>

                {/* Post options (ellipsis → Report abuse) */}
                <div className="relative -mr-2 flex justify-end">
                  <button
                    type="button"
                    aria-label="Post options"
                    aria-expanded={openMenu === item.id}
                    onClick={() => setOpenMenu((cur) => (cur === item.id ? null : item.id))}
                    className="inline-block px-4 py-1 text-center text-[rgb(129,129,129)] hover:text-[#111] dark:text-[var(--app-fg-muted)]"
                  >
                    <span className="flex h-4 w-4 items-center justify-center rounded-full border border-[rgb(129,129,129)] text-[10px] leading-none">
                      …
                    </span>
                  </button>
                  {openMenu === item.id && (
                    <ul
                      role="menu"
                      className="absolute right-0 top-full z-[30] mt-1 min-w-[180px] rounded-[8px] border border-black/5 bg-white p-2 text-left text-[14px] leading-[20px] shadow-[0_0_2px_rgba(0,0,0,0.15),0_2px_5px_rgba(0,0,0,0.05),0_8px_40px_rgba(0,0,0,0.04)] dark:bg-[var(--app-card)] dark:border-[var(--app-border)]"
                    >
                      <li>
                        <a
                          role="menuitem"
                          href="#"
                          onClick={(e) => {
                            e.preventDefault();
                            setOpenMenu(null);
                          }}
                          className="flex h-[38px] items-center gap-2 rounded-[8px] px-2 text-[14px] hover:bg-black/[0.04] dark:hover:bg-white/[0.06]"
                        >
                          <span className="w-[17.5px] text-center text-[14px]">⚑</span> Report abuse
                        </a>
                      </li>
                    </ul>
                  )}
                </div>
              </div>

              {/* Optional message bubble — mirrors builder inner border 1px solid #D4CFCD radius 18px */}
              {item.message && (
                <div className="rounded-[18px] border border-[rgb(212,207,205)] bg-white px-4 py-4 dark:bg-[var(--app-card)] dark:border-[var(--app-border)]">
                  <div className="whitespace-pre-line break-words px-0 py-0.5 text-[14px] leading-relaxed text-[#222] dark:text-[var(--app-fg)]">
                    {item.message}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Show more — mirrors builder-mtbc9yuo0hdig3ie5hze */}
        {canShowMore ? (
          <div className="mt-6 text-center">
            <button
              type="button"
              onClick={handleShowMore}
              className="inline text-[rgb(129,129,129)] underline decoration-[rgb(129,129,129)] underline-offset-2 transition-colors hover:text-[#111] dark:text-[var(--app-fg-muted)]"
            >
              Show more
            </button>
          </div>
        ) : (
          <p className="mt-6 text-center text-xs text-[rgb(129,129,129)] dark:text-[var(--app-fg-faint)]">
            You&apos;ve reached the end of the feed.
          </p>
        )}
      </div>

      {/* Click-away to close menus */}
      {openMenu && (
        <button
          type="button"
          aria-hidden
          tabIndex={-1}
          className="fixed inset-0 z-10 cursor-default bg-transparent"
          onClick={() => setOpenMenu(null)}
        />
      )}
    </div>
  );
}

// --------------------------------------------------------------- FeedsPage --
// Dashboard wrapper — use as the `element` for /dashboard or /feeds.
export function FeedsPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Dashboard"
        description="Live supporter feed — same layout as the Builder.io Feed Section, now as a native dashboard module (FeedsCard)."
      />
      <div className="grid gap-6 lg:grid-cols-[550px_1fr]">
        <FeedsCard />
        <div className="space-y-4">
          <Card className="p-5">
            <h2 className="text-sm font-semibold">About this module</h2>
            <p className="mt-1 text-sm leading-relaxed text-[var(--app-fg-muted)]">
              This card is a 1:1 React port of the supplied Builder JSON. Data lives in{" "}
              <code className="rounded bg-[var(--app-card-2)] px-1 py-0.5 font-mono text-xs">FEED_ITEMS</code> — swap it for
              a live API (<code className="font-mono text-xs">/api/feeds</code>) when you&apos;re ready.{" "}
              <code className="font-mono text-xs">openDonationModal()</code> now dispatches{" "}
              <code className="font-mono text-xs">gateway:openDonationModal</code> so you can wire your checkout without
              touching the card.
            </p>
            <ul className="mt-3 list-disc space-y-1 pl-5 text-sm text-[var(--app-fg-muted)]">
              <li>
                Import anywhere: <code className="font-mono text-xs">import {"{ FeedsCard }"} from &quot;@gateway/module-feeds&quot;</code>
              </li>
              <li>
                Dashboard route: <code className="font-mono text-xs">/dashboard</code> (also <code className="font-mono text-xs">/feeds</code>)
              </li>
              <li>
                Props: <code className="font-mono text-xs">items</code>, <code className="font-mono text-xs">pageSize</code>,{" "}
                <code className="font-mono text-xs">maxWidth</code>
              </li>
            </ul>
          </Card>
          <Card className="p-5">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--app-fg-muted)]">Next steps</h3>
            <p className="mt-2 text-sm text-[var(--app-fg-muted)]">
              Replace the static <code className="font-mono text-xs">FEED_ITEMS</code> with a fetch to your gateway and pass
              the result to <code className="font-mono text-xs">&lt;FeedsCard items=&#123;data&#125; /&gt;</code>. The card
              keeps the same rounded 18px, avatar 38px, and message-bubble styling from the original Builder export.
            </p>
          </Card>
        </div>
      </div>
    </div>
  );
}

// Keep DashboardPage alias for App.tsx clarity
export const DashboardPage = FeedsPage;
export default FeedsPage;
