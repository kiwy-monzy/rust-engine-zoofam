/** @gateway/module-auth — session state + login + guards + profile + sessions. */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import type { ReactNode } from "react";
import { Link, Navigate, Route, Routes, useLocation } from "react-router";
import {
  api,
  handleUnauthorized,
  session as store,
  ApiError,
} from "@gateway/lib";
import type {
  LoginResponse,
  MeResponse,
  ProfileAvatar,
  ProfileResponse,
  SessionUser,
  SessionView,
} from "@gateway/lib";
import {
  Alert,
  Badge,
  Button,
  Card,
  Field,
  Input,
  PageHeader,
  SidePanel,
  Spinner,
  Textarea,
} from "@gateway/ui";

// --------------------------------------------------------------- context --

interface SessionValue {
  user: SessionUser | null;
  roles: string[];
  permissions: string[];
  can: (perm: string) => boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refresh: () => Promise<void>;
}

const SessionContext = createContext<SessionValue | null>(null);

export function SessionProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<SessionUser | null>(() => store.user);
  const [roles, setRoles] = useState<string[]>(() => store.roles);
  const [permissions, setPermissions] = useState<string[]>(() => store.permissions);

  handleUnauthorized(() => {
    store.clear();
    setUser(null);
    setRoles([]);
    setPermissions([]);
  });

  const applyGrants = useCallback((me: MeResponse) => {
    setRoles(me.roles);
    setPermissions(me.permissions);
    store.setGrants(me.roles, me.permissions);
  }, []);

  const rehydrate = useCallback(async () => {
    if (!store.user) return;
    try {
      const me = await api.get<MeResponse>("/auth/me");
      applyGrants(me);
    } catch (e) {
      if (e instanceof ApiError && e.status === 401) {
        store.clear();
        setUser(null);
        setRoles([]);
        setPermissions([]);
      }
    }
  }, [applyGrants]);

  useEffect(() => {
    void rehydrate();
  }, [rehydrate]);

  const login = useCallback(
    async (email: string, password: string) => {
      const res = await api.post<LoginResponse>("/auth/login", { email, password });
      store.set(res.access_token, res.user);
      setUser(res.user);
      setRoles(res.roles ?? []);
      const flatPerms = Array.isArray(res.permissions)
        ? res.permissions.flatMap((p: any) =>
            Array.isArray(p.actions)
              ? p.actions.map((a: string) => `${p.module}:${a}`)
              : []
          )
        : [];
      setPermissions(flatPerms);
      store.setGrants(res.roles ?? [], flatPerms);
    },
    [],
  );

  const logout = useCallback(async () => {
    try {
      await api.post("/auth/logout");
    } catch (e) {
      if (!(e instanceof ApiError && (e.status === 401 || e.status === 0))) throw e;
    }
    store.clear();
    setUser(null);
    setRoles([]);
    setPermissions([]);
  }, []);

  const refresh = useCallback(async () => {
    await api.post("/auth/refresh");
  }, []);

  const value = useMemo<SessionValue>(
    () => ({
      user,
      roles,
      permissions,
      can: (perm) => permissions.includes(perm),
      login,
      logout,
      refresh,
    }),
    [user, roles, permissions, login, logout, refresh],
  );

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): SessionValue {
  const ctx = useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used inside <SessionProvider>");
  return ctx;
}

// ----------------------------------------------------------------- guard --

export function RequireAuth({ children }: { children: ReactNode }) {
  const { user } = useSession();
  const location = useLocation();
  if (!user) {
    const next = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?next=${next}`} replace />;
  }
  return <>{children}</>;
}

export function RequirePerm({ perm, children }: { perm: string; children: ReactNode }) {
  const { can, logout } = useSession();
  if (!can(perm)) {
    return (
      <div className="mx-auto max-w-md pt-16">
        <Alert tone="info">
          Your account lacks the <code className="font-mono">{perm}</code> permission. Ask an
          administrator to grant it — permissions are server-enforced and frozen into the
          token at issue time.
        </Alert>
        <div className="mt-3 flex justify-center">
          <Button variant="ghost" onClick={() => void logout()}>
            Sign in again
          </Button>
        </div>
      </div>
    );
  }
  return <>{children}</>;
}

// ---------------------------------------------------------------- shared --

const SIDE_IMAGE =
  "https://images.unsplash.com/photo-1505740420928-5e560c06d30e?w=1200&auto=format&fit=crop&q=80";

function GoogleIcon() {
  return (
    <svg className="mr-2 h-5 w-5" viewBox="0 0 24 24" aria-hidden>
      <path
        d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
        fill="#4285F4"
      />
      <path
        d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
        fill="#34A853"
      />
      <path
        d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
        fill="#FBBC05"
      />
      <path
        d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
        fill="#EA43334"
      />
    </svg>
  );
}

function AuthShell({
  children,
  imageAlt = "Stylish portrait with headphones and sunglasses",
}: {
  children: ReactNode;
  imageAlt?: string;
}) {
  return (
    <div className="min-h-screen w-full flex items-center justify-center p-4 md:p-6 lg:p-8 bg-gradient-to-br from-amber-100 via-orange-50 to-teal-100 dark:from-[#1a1a1a] dark:via-[#232323] dark:to-[#1a1a1a]">
      <div className="w-full max-w-[1000px] bg-white dark:bg-[var(--app-card)] rounded-2xl md:rounded-[2.5rem] shadow-2xl overflow-hidden border border-black/5 dark:border-white/10">
        <div className="grid lg:grid-cols-2 gap-0 min-h-[640px] lg:min-h-[680px]">
          <div className="flex flex-col items-center justify-center p-6 lg:p-10 bg-white dark:bg-[var(--app-card)]">
            {children}
          </div>
          <div className="relative lg:rounded-[2rem] m-0 lg:m-4 overflow-hidden min-h-[360px] lg:min-h-0 bg-gradient-to-br from-teal-300 via-cyan-200 to-orange-100">
            <img
              src={SIDE_IMAGE}
              alt={imageAlt}
              className="absolute inset-0 w-full h-full object-cover"
              loading="eager"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/15 via-transparent to-transparent" />
            <div className="absolute bottom-6 left-6 right-6 bg-white dark:bg-[var(--app-card)] rounded-2xl shadow-lg p-4 space-y-3">
              <p className="text-sm text-zinc-700 dark:text-[var(--app-fg)] leading-relaxed">
                Modern minimalist portrait featuring premium wireless headphones and sleek sunglasses against a gradient teal background.
              </p>
              <div className="flex items-center gap-2 flex-wrap">
                <button
                  type="button"
                  className="w-8 h-8 rounded-lg bg-zinc-100 dark:bg-[var(--app-card-2)] hover:bg-zinc-200 dark:hover:bg-white/10 flex items-center justify-center transition-colors"
                  aria-label="Add"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                  </svg>
                </button>
                <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-100 dark:bg-[var(--app-card-2)] rounded-lg">
                  <div className="w-2 h-2 bg-green-500 rounded-full" />
                  <span className="text-sm font-medium">Inspiration</span>
                  <svg className="w-4 h-4 ml-1 opacity-60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </div>
                <div className="flex items-center gap-2 px-3 py-1.5 bg-zinc-100 dark:bg-[var(--app-card-2)] rounded-lg">
                  <span className="text-sm font-medium">TableTop 1.0</span>
                  <svg className="w-4 h-4 opacity-60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </div>
                <button
                  type="button"
                  className="w-8 h-8 rounded-full bg-[var(--accent)] hover:bg-[var(--accent-strong)] text-white flex items-center justify-center transition-colors ml-auto"
                  aria-label="Go"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
                  </svg>
                </button>
              </div>
              <p className="text-xs text-zinc-400 dark:text-[var(--app-fg-faint)] text-center">Map & Fleet Platform · TableTop Labs</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ login --

export function LoginPage() {
  const { user, login } = useSession();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [resetSentTo, setResetSentTo] = useState<string | null>(null);
  const [resetOpen, setResetOpen] = useState(false);
  const [resetEmail, setResetEmail] = useState("");
  const [resetBusy, setResetBusy] = useState(false);
  const [resetError, setResetError] = useState<string | null>(null);
  const [resetDevToken, setResetDevToken] = useState<string | null>(null);
  const loc = useLocation();
  const next = new URLSearchParams(loc.search).get("next") ?? "/";

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await login(email.trim(), password);
    } catch (err) {
      setError(err instanceof Error ? err.message : "sign-in failed");
    } finally {
      setBusy(false);
    }
  }

  async function requestReset(e: React.FormEvent) {
    e.preventDefault();
    setResetBusy(true);
    setResetError(null);
    setResetDevToken(null);
    try {
      const res = await api.post<{ ok: boolean; dev_token?: string }>(
        "/auth/password-reset/request",
        { email: resetEmail.trim() },
      );
      setResetSentTo(resetEmail.trim());
      if (res.dev_token) setResetDevToken(res.dev_token);
    } catch (err) {
      setResetError(err instanceof Error ? err.message : "could not send reset email");
    } finally {
      setResetBusy(false);
    }
  }

  if (user) return <Navigate to={next} replace />;

  return (
    <AuthShell>
      <div className="w-full max-w-[420px] space-y-6">
        <div className="text-left">
          <div className="mb-3 inline-flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-[#d63232] to-[#b91c1c] font-bold text-white shadow">
            T
          </div>
          <h1 className="text-[32px] font-normal tracking-tight text-zinc-900 dark:text-white">Welcome back</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-[var(--app-fg-muted)]">Sign in to TableTop Labs</p>
        </div>

        <div className="space-y-4">
          <button
            type="button"
            onClick={() => setError("Google sign-in is not configured yet. Use email + password.")}
            className="w-full h-[50px] flex items-center justify-center bg-zinc-100 dark:bg-[var(--app-card-2)] hover:bg-zinc-200 dark:hover:bg-white/10 text-zinc-900 dark:text-white border border-zinc-200 dark:border-[var(--app-border)] rounded-xl font-normal text-[15px] transition-colors"
          >
            <GoogleIcon />
            Sign in with Google
          </button>

          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2 text-left">
              <label htmlFor="email" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
                Email
              </label>
              <input
                id="email"
                type="email"
                required
                autoComplete="username"
                placeholder="name@email.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
              />
            </div>

            <div className="space-y-2 text-left">
              <div className="flex items-center justify-between">
                <label htmlFor="password" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
                  Password
                </label>
                <button
                  type="button"
                  onClick={() => setResetOpen(true)}
                  className="text-[12px] text-zinc-500 dark:text-[var(--app-fg-muted)] hover:text-zinc-900 dark:hover:text-white"
                >
                  Forgot?
                </button>
              </div>
              <input
                id="password"
                type="password"
                required
                autoComplete="current-password"
                placeholder="••••••••"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
              />
            </div>

            {error && <Alert>{error}</Alert>}

            <Button type="submit" loading={busy} className="w-full h-[50px] rounded-xl text-[15px] font-normal">
              Sign in
            </Button>
          </form>

          <div className="text-center pt-1">
            <Link to="/register" className="text-[14px] font-normal text-zinc-500 dark:text-[var(--app-fg-muted)] hover:text-zinc-900 dark:hover:text-white transition-colors">
              Don&apos;t have an account? Create one
            </Link>
          </div>
        </div>
      </div>

      <SidePanel
        open={resetOpen}
        onClose={() => setResetOpen(false)}
        title="Reset password"
        width={420}
      >
        {resetSentTo ? (
          <div className="space-y-4">
            <Alert tone="success">
              If <strong>{resetSentTo}</strong> is registered, a reset link has been emailed.
              It expires in one hour.
            </Alert>
            {resetDevToken && (
              <Alert tone="info">
                <strong>Dev mode:</strong> SMTP is not configured, so the raw token is
                shown here for testing:
                <pre className="mt-2 break-all rounded bg-[var(--app-card-2)] p-2 text-xs">
                  {resetDevToken}
                </pre>
                <Link
                  to={`/reset-password?token=${encodeURIComponent(resetDevToken)}`}
                  className="mt-2 inline-block text-[var(--accent)] underline"
                >
                  Continue to reset form →
                </Link>
              </Alert>
            )}
            <div className="flex justify-end">
              <Button variant="ghost" onClick={() => setResetOpen(false)}>
                Close
              </Button>
            </div>
          </div>
        ) : (
          <form onSubmit={requestReset} className="space-y-4">
            <p className="text-sm text-[var(--app-fg-muted)]">
              Enter your email and we'll send a one-time link to choose a new password.
            </p>
            {resetError && <Alert tone="error">{resetError}</Alert>}
            <Field label="Email">
              <Input
                type="email"
                required
                placeholder="name@email.com"
                value={resetEmail}
                onChange={(e) => setResetEmail(e.target.value)}
              />
            </Field>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="ghost" onClick={() => setResetOpen(false)}>
                Cancel
              </Button>
              <Button type="submit" loading={resetBusy}>
                Send reset link
              </Button>
            </div>
          </form>
        )}
      </SidePanel>
    </AuthShell>
  );
}

// -------------------------------------------------------------- register --

const MIN_PASSWORD_LEN = 8;

export function RegisterPage() {
  const { user } = useSession();
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    if (password.length < MIN_PASSWORD_LEN) {
      setError(`the password must be at least ${MIN_PASSWORD_LEN} characters`);
      return;
    }
    if (password !== confirm) {
      setError("the passwords do not match");
      return;
    }
    setBusy(true);
    try {
      await api.post("/auth/register", {
        email: email.trim(),
        password,
        display_name: displayName.trim(),
      });
      setDone(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "registration failed");
    } finally {
      setBusy(false);
    }
  }

  if (user) return <Navigate to="/" replace />;

  if (done) {
    return (
      <AuthShell imageAlt="Account created">
        <div className="w-full max-w-[420px] space-y-6 text-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-emerald-100 text-emerald-600 dark:bg-emerald-900/30 dark:text-emerald-300">✓</div>
          <div>
            <h1 className="text-[28px] font-normal tracking-tight text-zinc-900 dark:text-white">Account created</h1>
            <p className="mt-2 text-sm text-zinc-500 dark:text-[var(--app-fg-muted)]">
              <strong className="font-medium text-zinc-900 dark:text-white">{email.trim()}</strong> is registered.
            </p>
          </div>
          <Alert tone="success">New accounts start with no roles. An admin will assign your role — then you can sign in.</Alert>
          <Link to="/login" replace>
            <Button className="w-full h-[50px] rounded-xl text-[15px]">Go to sign in</Button>
          </Link>
        </div>
      </AuthShell>
    );
  }

  return (
    <AuthShell>
      <div className="w-full max-w-[420px] space-y-6">
        <div className="text-left">
          <h1 className="text-[32px] font-normal tracking-tight text-zinc-900 dark:text-white">Create your account</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-[var(--app-fg-muted)]">Join TableTop Labs</p>
        </div>

        <div className="space-y-4">
          <button
            type="button"
            onClick={() => setError("Google sign-up is not configured yet. Use email + password.")}
            className="w-full h-[50px] flex items-center justify-center bg-zinc-100 dark:bg-[var(--app-card-2)] hover:bg-zinc-200 dark:hover:bg-white/10 text-zinc-900 dark:text-white border border-zinc-200 dark:border-[var(--app-border)] rounded-xl font-normal text-[15px] transition-colors"
          >
            <GoogleIcon />
            Sign up with Google
          </button>

          <form onSubmit={onSubmit} className="space-y-4">
            <div className="space-y-2 text-left">
              <label htmlFor="displayName" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
                Display name
              </label>
              <input
                id="displayName"
                type="text"
                autoComplete="name"
                placeholder="Ada Lovelace"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
              />
            </div>

            <div className="space-y-2 text-left">
              <label htmlFor="email" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
                Email
              </label>
              <input
                id="email"
                type="email"
                required
                autoComplete="username"
                placeholder="name@email.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
              />
            </div>

            <div className="space-y-2 text-left">
              <label htmlFor="password" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
                Password
              </label>
              <input
                id="password"
                type="password"
                required
                minLength={MIN_PASSWORD_LEN}
                autoComplete="new-password"
                placeholder={`at least ${MIN_PASSWORD_LEN} characters`}
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
              />
            </div>

            <div className="space-y-2 text-left">
              <label htmlFor="confirm" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
                Confirm password
              </label>
              <input
                id="confirm"
                type="password"
                required
                autoComplete="new-password"
                placeholder="repeat the password"
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
              />
            </div>

            {error && <Alert>{error}</Alert>}

            <Button type="submit" loading={busy} className="w-full h-[50px] rounded-xl text-[15px] font-normal">
              Create account
            </Button>
          </form>

          <div className="text-center pt-1 space-y-2">
            <Link to="/login" className="block text-[14px] font-normal text-zinc-500 dark:text-[var(--app-fg-muted)] hover:text-zinc-900 dark:hover:text-white transition-colors">
              Already have an account? Sign in
            </Link>
            <p className="text-xs text-zinc-400 dark:text-[var(--app-fg-faint)]">Registration grants no access until an admin assigns a role.</p>
          </div>
        </div>
      </div>
    </AuthShell>
  );
}

// --------------------------------------------------------------- reset --

export function ResetPasswordPage() {
  const loc = useLocation();
  const token = new URLSearchParams(loc.search).get("token") ?? "";
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    if (password.length < MIN_PASSWORD_LEN) {
      setError(`the password must be at least ${MIN_PASSWORD_LEN} characters`);
      return;
    }
    if (password !== confirm) {
      setError("the passwords do not match");
      return;
    }
    setBusy(true);
    try {
      await api.post("/auth/password-reset/confirm", { token, new_password: password });
      setDone(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "reset failed");
    } finally {
      setBusy(false);
    }
  }

  if (done) {
    return (
      <AuthShell imageAlt="Password reset">
        <div className="w-full max-w-[420px] space-y-6 text-center">
          <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-emerald-100 text-emerald-600 dark:bg-emerald-900/30 dark:text-emerald-300">✓</div>
          <h1 className="text-[28px] font-normal tracking-tight text-zinc-900 dark:text-white">Password updated</h1>
          <p className="text-sm text-zinc-500 dark:text-[var(--app-fg-muted)]">
            Your existing sessions were signed out — sign in with your new password.
          </p>
          <Link to="/login" replace>
            <Button className="w-full h-[50px] rounded-xl text-[15px]">Go to sign in</Button>
          </Link>
        </div>
      </AuthShell>
    );
  }

  if (!token) {
    return (
      <AuthShell imageAlt="Reset link invalid">
        <div className="w-full max-w-[420px] space-y-4 text-center">
          <h1 className="text-[28px] font-normal tracking-tight text-zinc-900 dark:text-white">Reset link invalid</h1>
          <p className="text-sm text-zinc-500 dark:text-[var(--app-fg-muted)]">
            Open the link from the email or request a new one from the sign-in page.
          </p>
          <Link to="/login" replace>
            <Button className="w-full h-[50px] rounded-xl text-[15px]">Back to sign in</Button>
          </Link>
        </div>
      </AuthShell>
    );
  }

  return (
    <AuthShell imageAlt="Reset your password">
      <div className="w-full max-w-[420px] space-y-6">
        <div className="text-left">
          <h1 className="text-[32px] font-normal tracking-tight text-zinc-900 dark:text-white">Choose a new password</h1>
          <p className="mt-1 text-sm text-zinc-500 dark:text-[var(--app-fg-muted)]">One-time link. Set a strong password.</p>
        </div>

        <form onSubmit={onSubmit} className="space-y-4">
          <div className="space-y-2 text-left">
            <label htmlFor="newpw" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
              New password
            </label>
            <input
              id="newpw"
              type="password"
              required
              minLength={MIN_PASSWORD_LEN}
              autoComplete="new-password"
              placeholder={`at least ${MIN_PASSWORD_LEN} characters`}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
            />
          </div>
          <div className="space-y-2 text-left">
            <label htmlFor="newpw2" className="text-[13px] font-normal text-zinc-700 dark:text-[var(--app-fg-muted)]">
              Confirm new password
            </label>
            <input
              id="newpw2"
              type="password"
              required
              autoComplete="new-password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              className="w-full h-[50px] bg-white dark:bg-[var(--app-card)] border border-zinc-200 dark:border-[var(--app-border)] rounded-xl px-3 text-[15px] text-zinc-900 dark:text-white placeholder:text-zinc-400 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/20 focus:border-[var(--accent)]"
            />
          </div>
          {error && <Alert>{error}</Alert>}
          <Button type="submit" loading={busy} className="w-full h-[50px] rounded-xl text-[15px] font-normal">
            Update password
          </Button>
        </form>
      </div>
    </AuthShell>
  );
}

// --------------------------------------------------------------- profile --

export function ProfilePage() {
  const { user, can } = useSession();
  const [profile, setProfile] = useState<ProfileResponse | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [savingProfile, setSavingProfile] = useState(false);
  const [pickingAvatar, setPickingAvatar] = useState<ProfileAvatar | null>(null);
  const [uploading, setUploading] = useState(false);
  const [uploadError, setUploadError] = useState<string | null>(null);
  const [changePwdOpen, setChangePwdOpen] = useState(false);
  const [changeBusy, setChangeBusy] = useState(false);
  const [changeError, setChangeError] = useState<string | null>(null);
  const [changeOk, setChangeOk] = useState(false);

  const [displayName, setDisplayName] = useState("");
  const [firstName, setFirstName] = useState("");
  const [middleName, setMiddleName] = useState("");
  const [lastName, setLastName] = useState("");
  const [username, setUsername] = useState("");
  const [phone, setPhone] = useState("");
  const [location, setLocation] = useState("");
  const [bio, setBio] = useState("");

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const p = await api.get<ProfileResponse>("/auth/profile");
      setProfile(p);
      setDisplayName(p.profile.display_name ?? p.user.display_name ?? "");
      setFirstName(p.user.first_name ?? "");
      setMiddleName(p.user.middle_name ?? "");
      setLastName(p.user.last_name ?? "");
      setUsername(p.user.username ?? "");
      setPhone(p.profile.phone ?? "");
      setLocation(p.profile.location ?? "");
      setBio(p.profile.bio ?? "");
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load profile");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function saveProfile(e: React.FormEvent) {
    e.preventDefault();
    setSavingProfile(true);
    setError(null);
    try {
      await api.patch("/auth/profile", {
        display_name: displayName || null,
        first_name: firstName || null,
        middle_name: middleName || null,
        last_name: lastName || null,
        username: username || null,
        phone: phone || null,
        location: location || null,
        bio: bio || null,
      });
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "could not save");
    } finally {
      setSavingProfile(false);
    }
  }

  async function uploadAvatar(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploading(true);
    setUploadError(null);
    try {
      await api.upload<{ file: ProfileAvatar }>("/auth/profile/avatar", file);
      await load();
    } catch (err) {
      setUploadError(err instanceof Error ? err.message : "upload failed");
    } finally {
      setUploading(false);
      e.target.value = "";
    }
  }

  async function pickAvatar(a: ProfileAvatar) {
    setPickingAvatar(a);
  }

  async function setPicked() {
    if (!pickingAvatar) return;
    setPickingAvatar(null);
    try {
      await api.post(`/auth/profile/avatar/pick/${pickingAvatar.id}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "could not set avatar");
    }
  }

  async function changePassword(e: React.FormEvent) {
    e.preventDefault();
    const form = e.currentTarget as HTMLFormElement;
    const data = new FormData(form);
    setChangeBusy(true);
    setChangeError(null);
    try {
      await api.post("/auth/profile/password", {
        current: String(data.get("current") ?? ""),
        new_password: String(data.get("new_password") ?? ""),
      });
      setChangeOk(true);
    } catch (err) {
      setChangeError(err instanceof Error ? err.message : "could not change password");
    } finally {
      setChangeBusy(false);
    }
  }

  if (!user) return <Navigate to="/login" replace />;
  if (!can("profile:read")) {
    return (
      <Alert tone="info">
        Your account lacks the <code className="font-mono">profile:read</code> permission.
      </Alert>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Profile"
        description="Your display info, current avatar, password and active sessions."
        actions={
          <Button onClick={() => setChangePwdOpen(true)}>Change password</Button>
        }
      />
      {error && <Alert tone="error">{error}</Alert>}
      {busy && !profile ? (
        <Spinner />
      ) : profile ? (
        <div className="grid gap-6 lg:grid-cols-2">
          <Card className="p-5 space-y-4">
            <div className="flex items-center gap-4">
              <AvatarView avatar={profile.current_avatar} fallbackEmail={user.email} />
              <div>
                <div className="text-sm font-semibold">{user.display_name || user.email}</div>
                <div className="text-xs text-[var(--app-fg-muted)]">{user.email}</div>
              </div>
            </div>
            <form onSubmit={saveProfile} className="space-y-3">
              <div className="grid grid-cols-2 gap-3">
                <Field label="First name">
                  <Input value={firstName} onChange={(e) => setFirstName(e.target.value)} placeholder="First name" />
                </Field>
                <Field label="Last name">
                  <Input value={lastName} onChange={(e) => setLastName(e.target.value)} placeholder="Last name" />
                </Field>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <Field label="Middle name">
                  <Input value={middleName} onChange={(e) => setMiddleName(e.target.value)} placeholder="Middle name" />
                </Field>
                <Field label="Username">
                  <Input value={username} onChange={(e) => setUsername(e.target.value)} placeholder="username" />
                </Field>
              </div>
              <Field label="Display name">
                <Input value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
              </Field>
              <div className="grid grid-cols-2 gap-3">
                <Field label="Phone">
                  <Input value={phone} onChange={(e) => setPhone(e.target.value)} />
                </Field>
                <Field label="Location">
                  <Input value={location} onChange={(e) => setLocation(e.target.value)} />
                </Field>
              </div>
              <Field label="Bio">
                <Textarea rows={3} value={bio} onChange={(e) => setBio(e.target.value)} />
              </Field>
              <div className="flex justify-end">
                <Button type="submit" loading={savingProfile}>Save profile</Button>
              </div>
            </form>
          </Card>

          <Card className="p-5 space-y-3">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold">Avatars</h3>
              <label className="cursor-pointer rounded-lg border border-[var(--app-border)] bg-[var(--app-card-2)] px-3 py-1.5 text-xs hover:bg-[var(--sb-hover)]">
                {uploading ? "Uploading…" : "Upload"}
                <input
                  type="file"
                  accept="image/png,image/jpeg,image/webp,image/gif"
                  className="hidden"
                  onChange={uploadAvatar}
                  disabled={uploading}
                />
              </label>
            </div>
            {uploadError && <Alert tone="error">{uploadError}</Alert>}
            {profile.avatars.length === 0 ? (
              <p className="text-sm text-[var(--app-fg-muted)]">
                No avatars uploaded yet. Upload one to set it as your profile picture.
              </p>
            ) : (
              <div className="grid grid-cols-4 gap-2">
                {profile.avatars.map((a) => (
                  <button
                    key={a.id}
                    type="button"
                    onClick={() => void pickAvatar(a)}
                    className={[
                      "relative aspect-square overflow-hidden rounded-lg border",
                      profile.current_avatar?.id === a.id
                        ? "border-[var(--accent)] ring-2 ring-[var(--accent)]"
                        : "border-[var(--app-border)] hover:border-[var(--accent)]",
                    ].join(" ")}
                    title={a.filename}
                  >
                    <AvatarView avatar={a} fallbackEmail={user.email} square />
                    {profile.current_avatar?.id === a.id && (
                      <span className="absolute right-1 top-1 rounded bg-[var(--accent)] px-1.5 text-[10px] text-white">
                        current
                      </span>
                    )}
                  </button>
                ))}
              </div>
            )}
          </Card>
        </div>
      ) : null}

      <SidePanel
        open={pickingAvatar !== null}
        onClose={() => setPickingAvatar(null)}
        title="Use this avatar?"
        width={360}
      >
        <div className="space-y-4">
          {pickingAvatar && (
            <AvatarView avatar={pickingAvatar} fallbackEmail={user.email} />
          )}
          <p className="text-sm text-[var(--app-fg-muted)]">
            This will be your profile picture across the app.
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setPickingAvatar(null)}>
              Cancel
            </Button>
            <Button onClick={() => void setPicked()}>Set as current</Button>
          </div>
        </div>
      </SidePanel>

      <SidePanel
        open={changePwdOpen}
        onClose={() => {
          setChangePwdOpen(false);
          setChangeOk(false);
          setChangeError(null);
        }}
        title="Change password"
        width={400}
      >
        {changeOk ? (
          <div className="space-y-3">
            <Alert tone="success">
              Your password was updated. All other sessions were signed out — sign in again.
            </Alert>
            <div className="flex justify-end">
              <Button
                onClick={() => {
                  setChangePwdOpen(false);
                  setChangeOk(false);
                  // Force a sign-out so the user re-authenticates with the new password.
                  void store.clear();
                  window.location.href = "/login";
                }}
              >
                Continue
              </Button>
            </div>
          </div>
        ) : (
          <form
            onSubmit={changePassword}
            className="space-y-4"
            onReset={() => setChangeError(null)}
          >
            {changeError && <Alert tone="error">{changeError}</Alert>}
            <Field label="Current password">
              <Input name="current" type="password" required autoComplete="current-password" />
            </Field>
            <Field label="New password">
              <Input
                name="new_password"
                type="password"
                required
                minLength={MIN_PASSWORD_LEN}
                autoComplete="new-password"
                placeholder={`at least ${MIN_PASSWORD_LEN} characters`}
              />
            </Field>
            <div className="flex justify-end gap-2">
              <Button type="button" variant="ghost" onClick={() => setChangePwdOpen(false)}>
                Cancel
              </Button>
              <Button type="submit" loading={changeBusy}>
                Update
              </Button>
            </div>
          </form>
        )}
      </SidePanel>
    </div>
  );
}

function AvatarView({
  avatar,
  fallbackEmail,
  square = false,
}: {
  avatar: ProfileAvatar | null;
  fallbackEmail: string;
  square?: boolean;
}) {
  const url = avatar
    ? `/api/v1/storage/${avatar.user_id}/${avatar.collection}/${encodeURIComponent(avatar.filename)}`
    : null;
  if (url) {
    return (
      <img
        src={url}
        alt="avatar"
        className={square ? "h-full w-full object-cover" : "h-16 w-16 rounded-full object-cover"}
      />
    );
  }
  return (
    <div
      className={[
        "grid place-items-center bg-[var(--app-card-2)] text-sm font-semibold text-[var(--app-fg-muted)]",
        square ? "h-full w-full" : "h-16 w-16 rounded-full",
      ].join(" ")}
    >
      {(fallbackEmail[0] ?? "?").toUpperCase()}
    </div>
  );
}

// --------------------------------------------------------------- sessions --

export function SessionsPage() {
  const { user } = useSession();
  const [sessions, setSessions] = useState<SessionView[] | null>(null);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const r = await api.get<{ sessions: SessionView[] }>("/auth/profile/sessions");
      setSessions(r.sessions);
    } catch (e) {
      setError(e instanceof Error ? e.message : "failed to load sessions");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
  }, []);

  async function revoke(id: string) {
    if (!confirm("Sign this session out?")) return;
    try {
      await api.del(`/auth/profile/sessions/${id}`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "could not revoke");
    }
  }

  if (!user) return <Navigate to="/login" replace />;

  return (
    <div className="space-y-6">
      <PageHeader
        title="Active sessions"
        description="Every device that has a valid session for your account. Revoking ends the session immediately."
        actions={<Button onClick={() => void load()}>Refresh</Button>}
      />
      {error && <Alert tone="error">{error}</Alert>}
      {busy ? (
        <Spinner />
      ) : sessions && sessions.length === 0 ? (
        <EmptyState msg="No active sessions." />
      ) : (
        <div className="space-y-3">
          {(sessions ?? []).map((s) => (
            <Card key={s.id} className="p-4">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-semibold">{s.device ?? "Unknown device"}</span>
                    {s.current && <Badge>current</Badge>}
                  </div>
                  <div className="text-xs text-[var(--app-fg-muted)]">
                    {s.ip ?? "?"} · created {new Date(s.created_at).toLocaleString()}
                  </div>
                  {s.user_agent && (
                    <div className="mt-1 line-clamp-1 max-w-2xl text-[10px] text-[var(--app-fg-faint)]">
                      {s.user_agent}
                    </div>
                  )}
                </div>
                <div className="flex gap-2">
                  {!s.current && (
                    <Button variant="danger" className="h-8 px-3 text-xs" onClick={() => void revoke(s.id)}>
                      Revoke
                    </Button>
                  )}
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}

function EmptyState({ msg }: { msg: string }) {
  return (
    <div className="rounded-lg border border-dashed border-[var(--app-border)] px-6 py-10 text-center text-sm text-[var(--app-fg-muted)]">
      {msg}
    </div>
  );
}

// --------------------------------------------------------------- routes --

export function AuthRoutes() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route path="/reset-password" element={<ResetPasswordPage />} />
    </Routes>
  );
}

export function ProfileRoutes() {
  return (
    <Routes>
      <Route path="/profile" element={<ProfilePage />} />
      <Route path="/profile/sessions" element={<SessionsPage />} />
    </Routes>
  );
}
