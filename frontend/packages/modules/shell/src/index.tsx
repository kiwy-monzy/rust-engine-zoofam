/**
 * @gateway/module-shell — ported from tauri-plugin-decorum (titlebar.js) and
 * tauri-plugin-sidebar (sidebar.js): the same 32px title bar and 188px/60px
 * collapsible navigation rail, themed #fafafa light / #1a1a1a dark with the
 * red (#d63232 → #b91c1c) accent.
 */

import { useCallback, useEffect, useState } from "react";
import type { ReactNode } from "react";
import { NavLink, useNavigate } from "react-router";
import { api } from "@gateway/lib";
import type { SystemInfo } from "@gateway/lib";

const cn = (...parts: Array<string | false | null | undefined>): string =>
  parts.filter(Boolean).join(" ");

// ------------------------------------------------------------------ icons --

const svg = (children: ReactNode, size = 16) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    {children}
  </svg>
);

export const icons = {
  panel: () =>
    svg(
      <>
        <rect x="3" y="3" width="18" height="18" rx="2" />
        <line x1="9" y1="3" x2="9" y2="21" />
      </>,
    ),
  search: () =>
    svg(
      <>
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </>,
      14,
    ),
  sun: () =>
    svg(
      <>
        <circle cx="12" cy="12" r="5" />
        <line x1="12" y1="1" x2="12" y2="3" />
        <line x1="12" y1="21" x2="12" y2="23" />
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
        <line x1="1" y1="12" x2="3" y2="12" />
        <line x1="21" y1="12" x2="23" y2="12" />
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
      </>,
      18,
    ),
  moon: () => svg(<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />, 18),
  logout: () =>
    svg(
      <>
        <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
        <polyline points="16 17 21 12 16 7" />
        <line x1="21" y1="12" x2="9" y2="12" />
      </>,
      18,
    ),
  users: () =>
    svg(
      <>
        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </>,
      18,
    ),
  shield: () => svg(<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />, 18),
  key: () =>
    svg(
      <>
        <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
      </>,
      18,
    ),
  wallet: () =>
    svg(
      <>
        <rect x="2" y="6" width="20" height="14" rx="2" />
        <path d="M2 10h20" />
        <path d="M6 15h4" />
      </>,
      18,
    ),
  gear: () =>
    svg(
      <>
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </>,
      18,
    ),
  lifebuoy: () =>
    svg(
      <>
        <circle cx="12" cy="12" r="10" />
        <circle cx="12" cy="12" r="4" />
        <line x1="4.93" y1="4.93" x2="9.17" y2="9.17" />
        <line x1="14.83" y1="14.83" x2="19.07" y2="19.07" />
        <line x1="14.83" y1="9.17" x2="19.07" y2="4.93" />
        <line x1="4.93" y1="19.07" x2="9.17" y2="14.83" />
      </>,
      18,
    ),
  map: () =>
    svg(
      <>
        <polygon points="1 6 1 22 8 18 16 22 23 18 23 2 16 6 8 2 1 6" />
        <line x1="8" y1="2" x2="8" y2="18" />
        <line x1="16" y1="6" x2="16" y2="22" />
      </>,
      18,
    ),
  folder: () =>
    svg(
      <>
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </>,
      18,
    ),
  fleet: () =>
    svg(
      <>
        <circle cx="12" cy="5" r="2" />
        <path d="M10 7h4l1 4h-6z" />
        <path d="M8 11l-2 8h12l-2-8" />
        <circle cx="7" cy="19" r="1" />
        <circle cx="17" cy="19" r="1" />
      </>,
      18,
    ),
  events: () =>
    svg(
      <>
        <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
      </>,
      18,
    ),
  feed: () =>
    svg(
      <>
        <path d="M4 11a9 9 0 0 1 9 9" />
        <path d="M4 4a16 16 0 0 1 16 16" />
        <circle cx="5" cy="19" r="1.5" />
      </>,
      18,
    ),
  dashboard: () =>
    svg(
      <>
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
      </>,
      18,
    ),
  commerce: () =>
    svg(
      <>
        <path d="M3 7l9-4 9 4-9 4-9-4z" />
        <path d="M3 7v6l9 4 9-4V7" />
        <path d="M3 13v4l9 4 9-4v-4" />
      </>,
      18,
    ),
  marketplace: () =>
    svg(
      <>
        <path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z" />
        <line x1="3" y1="6" x2="21" y2="6" />
        <path d="M16 10a4 4 0 0 1-8 0" />
      </>,
      18,
    ),
};

// ------------------------------------------------------------------- css --

const SHELL_CSS = `
.gw-sidebar {
  position: fixed; top: 32px; left: 0; z-index: 50;
  width: 188px; height: calc(100dvh - 32px);
  display: flex; flex-direction: column;
  background: var(--sb-bg);
  border-right: 1px solid var(--sb-border);
  overflow: clip;
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: width;
}
.gw-sidebar.collapsed { width: 60px; }
.gw-sidebar .sb-scroll { flex: 1; overflow-y: auto; overflow-x: hidden; padding: 6px 6px 4px; }
.gw-sidebar .sb-nav { display: flex; flex-direction: column; gap: 2px; }
.gw-sidebar .sb-nav-item {
  display: flex; align-items: center; gap: 10px;
  padding: 6px 8px; border-radius: 10px; cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease,
    padding 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  text-decoration: none; position: relative;
  border: none; background: none; width: 100%; color: var(--sb-fg);
}
.gw-sidebar .sb-nav-item:hover { background: var(--sb-hover); color: var(--sb-fg); }
.gw-sidebar .sb-nav-item.active {
  background: var(--sb-active-bg); color: var(--sb-active-fg);
  border-radius: 12px;
}
.gw-sidebar.collapsed .sb-nav-item { padding: 6px 6px; }
.gw-sidebar .sb-icon-box {
  width: 36px; height: 36px; display: flex; align-items: center; justify-content: center;
  border-radius: 12px; flex-shrink: 0;
  transition: background 0.15s ease, color 0.15s ease, box-shadow 0.2s ease, transform 0.2s ease;
  color: var(--sb-fg); background: var(--sb-icon-bg);
  border: 1px solid var(--sb-icon-border);
  box-shadow: var(--sb-icon-shadow);
}
.gw-sidebar .sb-nav-item:hover .sb-icon-box {
  color: var(--sb-fg); background: var(--sb-hover);
  box-shadow: var(--sb-icon-hover-shadow);
}
.gw-sidebar .sb-nav-item.active .sb-icon-box {
  background: linear-gradient(145deg, #d63232, #b91c1c); color: #fff;
  border-color: rgba(0,0,0,0.3);
  box-shadow: inset 2px 2px 4px rgba(0,0,0,0.3), inset -2px -2px 4px rgba(255,255,255,0.1);
  transform: scale(0.95);
}
.gw-sidebar .sb-label {
  flex: 1; min-width: 0;
  font-size: 13px; font-weight: 500; white-space: nowrap;
  overflow: hidden; text-overflow: ellipsis;
  opacity: 1; transition: opacity 0.12s ease;
}
.gw-sidebar.collapsed .sb-label { opacity: 0; pointer-events: none; }
.gw-sidebar .sb-nav-item.active .sb-label { font-weight: 600; }
.gw-sidebar .sb-group { margin-bottom: 8px; }
.gw-sidebar .sb-group-label {
  font-size: 10px; font-weight: 700; text-transform: uppercase;
  letter-spacing: 0.08em; color: var(--sb-fg-muted);
  padding: 8px 8px 4px; opacity: 0.7;
}
.gw-sidebar.collapsed .sb-group-label { display: none; }
.gw-sidebar .sb-footer {
  flex-shrink: 0; padding: 6px 6px 10px; display: flex; flex-direction: column; gap: 2px;
  border-top: 1px solid var(--sb-footer-border);
}
.gw-sidebar .sb-footer-btn {
  display: flex; align-items: center; justify-content: flex-start; gap: 10px; width: 100%;
  text-align: left;
  padding: 6px 8px; border-radius: 10px; border: none; background: none;
  color: var(--sb-fg-muted); font-size: 13px; font-weight: 500; cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease,
    padding 0.2s cubic-bezier(0.4, 0, 0.2, 1); position: relative;
}
.gw-sidebar .sb-footer-btn:hover { color: var(--sb-fg); background: var(--sb-hover); }
.gw-sidebar .sb-footer-btn .sb-icon-box { width: 32px; height: 32px; border-radius: 10px; }
.gw-sidebar .sb-footer-btn.logout:hover { background: rgba(220,38,38,0.9); color: #fff; }
.gw-sidebar.collapsed .sb-footer-btn { padding: 6px 8px; }
.gw-sidebar .sb-version {
  text-align: center; font-size: 10px; color: var(--sb-version-fg);
  padding: 4px 0; font-family: monospace;
  display: flex; flex-direction: column; gap: 1px;
}
.gw-sidebar .sb-version-name { font-weight: 600; font-size: 11px; }
.gw-sidebar .sb-version-sub { font-size: 9px; opacity: 0.7; }
.gw-sidebar .sb-version-num { font-size: 9px; opacity: 0.5; }
.gw-hovertip {
  position: fixed; z-index: 2000; left: 0; top: 0; padding: 5px 10px;
  background: #dc2626; color: #fff; font-size: 12px; font-weight: 600;
  border-radius: 6px; white-space: nowrap; pointer-events: none; opacity: 0;
  transform: translateY(-50%); transition: opacity 0.12s ease;
  box-shadow: 0 4px 12px rgba(0,0,0,0.35);
}
.gw-titlebar {
  position: fixed; top: 0; left: 0; right: 0; z-index: 100; height: 32px;
  display: flex; align-items: center;
  background: var(--sb-bg);
  user-select: none;
}
/* Bottom rule starts at the sidebar's edge so the sidebar's right border and
   this rule meet in one clean corner instead of crossing. */
.gw-titlebar::after {
  content: ""; position: absolute; top: 31px; left: var(--gw-sb-w, 188px); right: 0;
  height: 1px; background: var(--sb-border); pointer-events: none;
  transition: left 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.gw-titlebar .tb-left {
  display: flex; align-items: center; gap: 6px; height: 100%;
  padding: 0 10px; flex-shrink: 0; pointer-events: none;
}
.gw-titlebar .tb-logo {
  width: 18px; height: 18px; border-radius: 5px; pointer-events: none;
  display: flex; align-items: center; justify-content: center;
  background: linear-gradient(145deg, #d63232, #b91c1c);
  color: #fff; font-size: 11px; font-weight: 700;
}
.gw-titlebar .tb-logo-img {
  width: 18px; height: 18px; border-radius: 5px; pointer-events: none;
  object-fit: cover; flex-shrink: 0;
}
.gw-titlebar .tb-name { font-size: 12px; font-weight: 500; color: var(--sb-fg-muted); pointer-events: none; }
.gw-titlebar .tb-tools {
  display: flex; align-items: center; gap: 8px; height: 100%;
  padding: 0 6px; flex-shrink: 0; pointer-events: auto;
}
.gw-titlebar .tb-btn {
  display: flex; align-items: center; justify-content: center;
  width: 26px; height: 26px; border: none; background: none;
  color: var(--sb-fg-muted); border-radius: 6px; cursor: pointer;
  transition: all 0.15s ease;
}
.gw-titlebar .tb-btn:hover { background: var(--sb-hover); color: var(--sb-fg); }
.gw-titlebar .tb-search { position: relative; width: 300px; max-width: 34vw; }
.gw-titlebar .tb-search-inner {
  display: flex; align-items: center; gap: 6px; height: 24px; padding: 0 8px;
  border: 1px solid var(--sb-border); border-radius: 8px;
  background: var(--app-card);
}
.gw-titlebar .tb-search-inner svg { flex-shrink: 0; color: var(--sb-fg-muted); }
.gw-titlebar .tb-search-inner input {
  flex: 1; min-width: 0; border: none; background: none; outline: none;
  font-size: 12px; color: var(--app-fg);
}
.gw-titlebar .tb-search-inner input::placeholder { color: var(--sb-fg-muted); }
.gw-titlebar .tb-drag { flex: 1; height: 100%; }
.gw-titlebar .tb-right {
  display: flex; align-items: center; gap: 6px; height: 100%;
  padding: 0 10px 0 8px; flex-shrink: 0;
}
.gw-titlebar .tb-avatar {
  width: 22px; height: 22px; border-radius: 50%; object-fit: cover; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  background: linear-gradient(145deg, #d63232, #b91c1c);
  color: #fff; font-size: 10px; font-weight: 700;
}
.gw-titlebar .tb-username {
  font-size: 11px; font-weight: 500; max-width: 110px;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  color: var(--sb-fg);
}

/* ── mobile ≤ 768px: sidebar → bottom tab bar ─────────────────────── */
@media (max-width: 768px) {
  .gw-sidebar {
    top: auto; bottom: 0; left: 0; right: 0;
    width: 100%; height: auto; max-height: 60px;
    flex-direction: row; border-right: none;
    border-top: 1px solid var(--sb-border);
    z-index: 150;
  }
  .gw-sidebar .sb-scroll {
    flex: 1; overflow-x: auto; overflow-y: hidden;
    display: flex; padding: 4px 6px; gap: 0;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
  }
  .gw-sidebar .sb-scroll::-webkit-scrollbar { display: none; }
  .gw-sidebar .sb-nav {
    flex-direction: row; gap: 0; flex-shrink: 0;
  }
  .gw-sidebar .sb-nav-item {
    width: auto; flex-shrink: 0;
    flex-direction: column; gap: 2px;
    padding: 6px 12px; min-width: 56px;
    justify-content: center; align-items: center;
  }
  .gw-sidebar .sb-nav-item.active { border-radius: 10px; }
  .gw-sidebar .sb-icon-box { width: 32px; height: 32px; border-radius: 10px; }
  .gw-sidebar .sb-nav-item.active .sb-icon-box { transform: scale(0.95); }
  .gw-sidebar .sb-label {
    font-size: 9px; font-weight: 500; opacity: 1;
    white-space: nowrap; overflow: visible; text-overflow: unset;
    min-width: 0; flex: unset;
  }
  .gw-sidebar.collapsed .sb-label { opacity: 1; pointer-events: auto; }
  .gw-sidebar .sb-footer { display: none; }
  .gw-sidebar.collapsed { width: 100%; }

  /* titlebar */
  .gw-titlebar { z-index: 200; }
  .gw-titlebar::after { left: 0 !important; }
  .gw-titlebar .tb-left .tb-name { display: none; }
  .gw-titlebar .tb-tools .tb-btn-sidebar { display: none; }
  .gw-titlebar .tb-search { width: 140px; max-width: 40vw; }
  .gw-titlebar .tb-username { display: none; }

  /* hovertip disabled on mobile */
  .gw-hovertip { display: none !important; }
}

/* ── main content offset for mobile bottom bar ────────────────────── */
@media (max-width: 768px) {
  .gw-main-wrap { margin-left: 0 !important; padding-bottom: 64px !important; }
}`;

function ensureShellCss() {
  if (!document.getElementById("gw-shell-css")) {
    const el = document.createElement("style");
    el.id = "gw-shell-css";
    el.textContent = SHELL_CSS;
    document.head.appendChild(el);
  }
}

// ------------------------------------------------------------------ theme --

const THEME_KEY = "gateway.theme";

export function initTheme(): boolean {
  let dark = false;
  try {
    const stored = localStorage.getItem(THEME_KEY);
    dark = stored
      ? stored === "dark"
      : window.matchMedia("(prefers-color-scheme: dark)").matches;
  } catch {
    dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  document.documentElement.classList.toggle("dark", dark);
  return dark;
}

export function useTheme() {
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark"),
  );

  useEffect(() => {
    initTheme();
    setDark(document.documentElement.classList.contains("dark"));
  }, []);

  const toggle = useCallback(() => {
    const next = !document.documentElement.classList.contains("dark");
    document.documentElement.classList.toggle("dark", next);
    try {
      localStorage.setItem(THEME_KEY, next ? "dark" : "light");
    } catch {
      /* private mode */
    }
    setDark(next);
  }, []);

  return { dark, toggle };
}

// ---------------------------------------------------------------- sidebar --

export interface SidebarItem {
  to: string;
  label: string;
  icon: ReactNode;
  perm?: string;
}

export interface SidebarGroup {
  label: string;
  items: SidebarItem[];
}

export function Sidebar({
  items,
  collapsed,
  onLogout,
  groups,
}: {
  items?: SidebarItem[];
  collapsed: boolean;
  onLogout: () => void | Promise<void>;
  groups?: SidebarGroup[];
}) {
  ensureShellCss();
  const { dark, toggle } = useTheme();
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  useEffect(() => {
    api
      .get<{ system: SystemInfo }>("/system")
      .then((res) => setSystemInfo(res.system))
      .catch(() => {});
  }, []);

  function tipHandlers(text: string) {
    return {
      onMouseEnter: (e: React.MouseEvent<HTMLElement>) => {
        if (!collapsed) return;
        const r = e.currentTarget.getBoundingClientRect();
        let tip = document.getElementById("gw-hovertip");
        if (!tip) {
          tip = document.createElement("div");
          tip.id = "gw-hovertip";
          tip.className = "gw-hovertip";
          document.body.appendChild(tip);
        }
        tip.textContent = text;
        tip.style.left = `${r.right + 8}px`;
        tip.style.top = `${r.top + r.height / 2}px`;
        tip.style.opacity = "1";
      },
      onMouseLeave: () => {
        const tip = document.getElementById("gw-hovertip");
        if (tip) tip.style.opacity = "0";
      },
    };
  }

  const renderItems = (itemList: SidebarItem[]) => (
    <nav className="sb-nav">
      {itemList.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          end
          {...tipHandlers(item.label)}
          className={({ isActive }) =>
            cn("sb-nav-item", isActive && "active")
          }
        >
          <span className="sb-icon-box">{item.icon}</span>
          <span className="sb-label">{item.label}</span>
        </NavLink>
      ))}
    </nav>
  );

  return (
    <aside className={cn("gw-sidebar", collapsed && "collapsed")}>
      <div className="sb-scroll">
        {groups
          ? groups.map((group) => (
              <div key={group.label} className="sb-group">
                {!collapsed && <div className="sb-group-label">{group.label}</div>}
                {renderItems(group.items)}
              </div>
            ))
          : items && renderItems(items)}
      </div>

      <div className="sb-footer">
        <button type="button" className="sb-footer-btn" onClick={toggle} {...tipHandlers("Theme")}>
          <span className="sb-icon-box">{dark ? <icons.sun /> : <icons.moon />}</span>
          <span className="sb-label">Theme</span>
        </button>
        <button
          type="button"
          className="sb-footer-btn logout"
          onClick={() => void onLogout()}
          {...tipHandlers("Log out")}
        >
          <span className="sb-icon-box"><icons.logout /></span>
          <span className="sb-label">Log out</span>
        </button>
        <div className="sb-version">
          <span className="sb-version-name">{systemInfo?.name || "TableTop Labs"}</span>
          <span className="sb-version-sub">Map &amp; Fleet Platform</span>
          <span className="sb-version-num">v{systemInfo?.version || "0.1.0"}</span>
        </div>
      </div>
    </aside>
  );
}

// --------------------------------------------------------------- titlebar --

export function TitleBar({
  collapsed,
  onToggleSidebar,
  userName,
  systemName,
  logoUrl,
}: {
  collapsed: boolean;
  onToggleSidebar: () => void;
  userName?: string;
  systemName?: string;
  logoUrl?: string;
}) {
  ensureShellCss();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Array<{ id: string; kind: string; name: string; description: string; url?: string }>>([]);
  const [searching, setSearching] = useState(false);
  const [showResults, setShowResults] = useState(false);

  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  function handleSearch(value: string) {
    setQuery(value);
    if (debounceTimer) clearTimeout(debounceTimer);
    if (!value.trim()) {
      setResults([]);
      setShowResults(false);
      return;
    }
    debounceTimer = setTimeout(async () => {
      setSearching(true);
      try {
        const res = await api.get<{ results: Array<{ id: string; kind: string; name: string; description: string; url?: string }> }>(
          `/search?q=${encodeURIComponent(value.trim())}&limit=12`
        );
        setResults(res.results);
        setShowResults(true);
      } catch {
        setResults([]);
      } finally {
        setSearching(false);
      }
    }, 250);
  }

  function handleResultClick(r: { kind: string; url?: string; name: string }) {
    setShowResults(false);
    setQuery("");
    if (r.kind === "layer") navigate("/maps");
    else if (r.kind === "source") navigate("/maps");
    else if (r.kind === "file" && r.url) window.open(r.url, "_blank");
    else navigate("/maps");
  }

  function submit(e: React.FormEvent) {
    e.preventDefault();
    if (results.length > 0) {
      handleResultClick(results[0]);
    } else {
      navigate(`/maps${query.trim() ? `?q=${encodeURIComponent(query.trim())}` : ""}`);
    }
  }

  return (
    <header
      className="gw-titlebar"
      style={{ "--gw-sb-w": collapsed ? "60px" : "188px" } as React.CSSProperties}
    >
      <div className="tb-left">
        {logoUrl ? (
          <img className="tb-logo-img" src={logoUrl} alt="" />
        ) : (
          <span className="tb-logo">T</span>
        )}
        <span className="tb-name">{systemName || "TableTop Labs"}</span>
      </div>

      <div className="tb-tools">
        <button
          type="button"
          className="tb-btn tb-btn-sidebar"
          title="Toggle sidebar"
          aria-pressed={collapsed}
          onClick={onToggleSidebar}
        >
          <icons.panel />
        </button>
        <div className="tb-search">
          <form className="tb-search-inner" onSubmit={submit}>
            <icons.search />
            <input
              type="text"
              placeholder="Search layers, files…"
              value={query}
              onChange={(e) => handleSearch(e.target.value)}
              onFocus={() => results.length > 0 && setShowResults(true)}
              onBlur={() => setTimeout(() => setShowResults(false), 200)}
            />
            {searching && <span className="text-[10px] text-[var(--sb-fg-muted)]">…</span>}
          </form>
          {showResults && results.length > 0 && (
            <div className="absolute left-0 top-full mt-1 w-full rounded-lg border border-[var(--app-border)] bg-[var(--app-card)] shadow-lg z-50 max-h-80 overflow-y-auto">
              {results.map((r) => (
                <button
                  key={r.id}
                  type="button"
                  className="flex w-full items-start gap-2 px-3 py-2 text-left text-sm hover:bg-[var(--app-card-2)] transition-colors"
                  onMouseDown={() => handleResultClick(r)}
                >
                  <span className="mt-0.5 inline-flex h-4 w-4 shrink-0 items-center justify-center rounded bg-[var(--app-card-2)] text-[10px] font-bold text-[var(--app-fg-muted)]">
                    {r.kind === "layer" ? "L" : r.kind === "source" ? "S" : "F"}
                  </span>
                  <span className="min-w-0">
                    <span className="block truncate font-medium text-[var(--app-fg)]">{r.name}</span>
                    <span className="block truncate text-xs text-[var(--app-fg-muted)]">{r.description}</span>
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="tb-drag" data-tauri-drag-region="" />

      <div className="tb-right" title={userName}>
        <span className="tb-avatar">{(userName || "?").charAt(0).toUpperCase()}</span>
        <span className="tb-username">{userName}</span>
      </div>
    </header>
  );
}
