import { useState } from "react";
import { Navigate, Route, Routes, Outlet, useLocation, useParams } from "react-router";
import { cn } from "@gateway/ui";
import {
  LoginPage,
  RegisterPage,
  ResetPasswordPage,
  RequireAuth,
  RequirePerm,
  SessionProvider,
  useSession,
  ProfilePage,
  SessionsPage,
} from "@gateway/module-auth";
import {
  RbacLayout,
  UsersPage,
  RolesPage,
  PermissionsPage,
} from "@gateway/module-rbac";
import { SupportPage } from "@gateway/module-support";
import { SystemPage, ReleasesPage } from "@gateway/module-system";
import { MapsPage } from "@gateway/module-maps";
import StoragePage from "@gateway/module-storage";
import FleetPage from "@gateway/module-fleet";
import { FeedsPage } from "@gateway/module-feeds";
import {
  ErpLayout,
  ErpOverview,
  ProductsPage,
  UnitsPage,
  WarehousesPage,
  SuppliersPage,
  ProcurementPage,
  PurchaseOrdersPage,
  GoodsReceiptsPage,
  InventoryPage,
  SalesOrdersPage,
  ShipmentsPage,
  AssetsPage,
  ExpensesPage,
  InvoicesPage,
} from "@gateway/module-erp";
import {
  CrmLayout,
  CrmOverview,
  LeadsPage,
  OpportunitiesPage,
  CustomersPage,
  ContactsPage,
  ActivitiesPage,
  QuotesPage as CrmQuotesPage,
} from "@gateway/module-crm";
import {
  DmcLayout,
  DmcOverview,
  DestinationsPage,
  PackagesPage,
  ActivitiesPage as DmcActivitiesPage,
  AccommodationPage,
  TransportPage,
  GuidesPage,
  TripsPage,
  SupplierCategoriesPage,
  ItinerariesPage,
  QuotesPage as DmcQuotesPage,
  BookingsPage as DmcBookingsPage,
  CatalogPage,
  IncidentsPage,
  TripDetailPage,
  WalletPage,
} from "@gateway/module-dmc";
import {
  MarketplaceLayout,
  MarketplaceOverview,
  OrganizationsPage,
  OrganizationDetailPage,
  CategoriesPage,
  ServicesPage,
  BookingsPage,
  RequestsPage,
  PromotionsPage,
  NotificationsPage,
  PayoutsPage,
} from "@gateway/module-marketplace";
import {
  WebsiteLayout,
  WebsiteOverview,
  HeroPage,
  SlideshowPage,
  DownloadsPage,
  LinksPage,
  ProductsPage as WebsiteProductsPage,
  CartPage,
  BookingsPage as WebsiteBookingsPage,
  ThemePage,
  PreviewPage,
} from "@gateway/module-website";
import { Sidebar, TitleBar, icons } from "@gateway/module-shell";
import type { SidebarGroup } from "@gateway/module-shell";

const SIDEBAR_KEY = "gateway.sidebar.collapsed";

const NAV: SidebarGroup[] = [
  {
    label: "Overview",
    items: [
      { to: "/dashboard", label: "Dashboard", icon: <icons.dashboard /> },
    ],
  },
  {
    label: "Business",
    items: [
      { to: "/website", label: "Website", icon: <icons.panel />, perm: "website:read" },
      { to: "/erp", label: "ERP", icon: <icons.commerce />, perm: "erp:read" },
      { to: "/crm", label: "CRM", icon: <icons.users />, perm: "crm:read" },
      { to: "/dmc", label: "DMC", icon: <icons.commerce />, perm: "dmc:read" },
      { to: "/marketplace", label: "Marketplace", icon: <icons.marketplace />, perm: "marketplace:read" },
    ],
  },
  {
    label: "Access Control",
    items: [
      { to: "/rbac/users", label: "Users", icon: <icons.users />, perm: "users:read" },
      { to: "/rbac/roles", label: "Roles", icon: <icons.shield />, perm: "roles:read" },
      { to: "/rbac/permissions", label: "Permissions", icon: <icons.key />, perm: "permissions:read" },
    ],
  },
  {
    label: "Operations",
    items: [
      { to: "/fleet", label: "Fleet", icon: <icons.fleet />, perm: "fleet:read" },
      { to: "/maps", label: "Maps", icon: <icons.map />, perm: "maps:read" },
      { to: "/storage", label: "Storage", icon: <icons.folder />, perm: "storage:read" },
      { to: "/support", label: "Support", icon: <icons.lifebuoy />, perm: "support:read" },
    ],
  },
  {
    label: "Account",
    items: [
      { to: "/profile", label: "Profile", icon: <icons.users />, perm: "profile:read" },
      { to: "/profile/sessions", label: "Sessions", icon: <icons.fleet />, perm: "profile:read" },
    ],
  },
  {
    label: "System",
    items: [
      { to: "/system", label: "Settings", icon: <icons.gear /> },
      { to: "/system/releases", label: "Releases", icon: <icons.folder />, perm: "releases:read" },
    ],
  },
];

export default function App() {
  return (
    <SessionProvider>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/reset-password" element={<ResetPasswordPage />} />
        <Route
          element={
            <RequireAuth>
              <Layout />
            </RequireAuth>
          }
        >
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<FeedsPage />} />
          <Route path="feeds" element={<FeedsPage />} />

          {/* RBAC — Users, Roles, Permissions in one module */}
          <Route path="rbac" element={<RbacLayout />}>
            <Route index element={<Navigate to="users" replace />} />
            <Route path="users" element={<RbacUsersRoute />} />
            <Route path="roles" element={<RbacRolesRoute />} />
            <Route path="permissions" element={<RbacPermissionsRoute />} />
          </Route>

          {/* ERP */}
          <Route path="erp" element={<ErpLayout />}>
            <Route index element={<ErpOverview />} />
            <Route path="products" element={<ProductsPage />} />
            <Route path="units" element={<UnitsPage />} />
            <Route path="warehouses" element={<WarehousesPage />} />
            <Route path="suppliers" element={<SuppliersPage />} />
            <Route path="procurement" element={<ProcurementPage />} />
            <Route path="purchase-orders" element={<PurchaseOrdersPage />} />
            <Route path="goods-receipts" element={<GoodsReceiptsPage />} />
            <Route path="inventory" element={<InventoryPage />} />
            <Route path="sales-orders" element={<SalesOrdersPage />} />
            <Route path="shipments" element={<ShipmentsPage />} />
            <Route path="assets" element={<AssetsPage />} />
            <Route path="expenses" element={<ExpensesPage />} />
            <Route path="invoices" element={<InvoicesPage />} />
          </Route>

          {/* Legacy aliases */}
          <Route path="commerce" element={<Navigate to="/erp" replace />} />
          <Route path="commerce/*" element={<Navigate to="/erp" replace />} />
          <Route path="erp/crm/*" element={<Navigate to="/crm" replace />} />

          {/* CRM */}
          <Route path="crm" element={<CrmLayout />}>
            <Route index element={<CrmOverview />} />
            <Route path="leads" element={<LeadsPage />} />
            <Route path="opportunities" element={<OpportunitiesPage />} />
            <Route path="customers" element={<CustomersPage />} />
            <Route path="contacts" element={<ContactsPage />} />
            <Route path="activities" element={<ActivitiesPage />} />
            <Route path="quotes" element={<CrmQuotesPage />} />
          </Route>

          {/* DMC */}
          <Route path="dmc" element={<DmcLayout />}>
            <Route index element={<DmcOverview />} />
            <Route path="destinations" element={<DestinationsPage />} />
            <Route path="packages" element={<PackagesPage />} />
            <Route path="packages/:pkgId/itineraries" element={<ItinerariesPage />} />
            <Route path="activities" element={<DmcActivitiesPage />} />
            <Route path="accommodation" element={<AccommodationPage />} />
            <Route path="transport" element={<TransportPage />} />
            <Route path="guides" element={<GuidesPage />} />
            <Route path="trips" element={<TripsPage />} />
            <Route path="trips/:id" element={<TripDetailPage />} />
            <Route path="quotes" element={<DmcQuotesPage />} />
            <Route path="bookings" element={<DmcBookingsPage />} />
            <Route path="catalog" element={<CatalogPage />} />
            <Route path="incidents" element={<IncidentsPage />} />
            <Route path="supplier-categories" element={<SupplierCategoriesPage />} />
            <Route path="wallet" element={<WalletPage />} />
          </Route>

          {/* Marketplace */}
          <Route path="marketplace" element={<MarketplaceLayout />}>
            <Route index element={<MarketplaceOverview />} />
            <Route path="organizations" element={<OrganizationsPage />} />
            <Route path="organizations/:id" element={<OrgDetailWrapper />} />
            <Route path="categories" element={<CategoriesPage />} />
            <Route path="services" element={<ServicesPage />} />
            <Route path="bookings" element={<BookingsPage />} />
            <Route path="requests" element={<RequestsPage />} />
            <Route path="promotions" element={<PromotionsPage />} />
            <Route path="notifications" element={<NotificationsPage />} />
            <Route path="payouts" element={<PayoutsPage />} />
          </Route>

          {/* Website */}
          <Route path="website" element={<WebsiteLayout />}>
            <Route index element={<WebsiteOverview />} />
            <Route path="hero" element={<HeroPage />} />
            <Route path="slideshow" element={<SlideshowPage />} />
            <Route path="downloads" element={<DownloadsPage />} />
            <Route path="links" element={<LinksPage />} />
            <Route path="products" element={<WebsiteProductsPage />} />
            <Route path="cart" element={<CartPage />} />
            <Route path="bookings" element={<WebsiteBookingsPage />} />
            <Route path="theme" element={<ThemePage />} />
            <Route path="preview" element={<PreviewPage />} />
          </Route>

          {/* Legacy RBAC routes redirect */}
          <Route path="users" element={<Navigate to="/rbac/users" replace />} />
          <Route path="roles" element={<Navigate to="/rbac/roles" replace />} />
          <Route path="permissions" element={<Navigate to="/rbac/permissions" replace />} />

          {/* System with sub-modules */}
          <Route path="system" element={<SystemLayout />}>
            <Route index element={<SystemSettingsRoute />} />
            <Route path="releases" element={<ReleasesRoute />} />
          </Route>

          {/* Standalone routes */}
          <Route path="maps" element={<MapsRoute />} />
          <Route path="fleet" element={<FleetRoute />} />
          <Route path="storage" element={<StorageRoute />} />
          <Route path="support" element={<SupportRoute />} />
          <Route path="profile" element={<ProfilePage />} />
          <Route path="profile/sessions" element={<SessionsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </SessionProvider>
  );
}

function RbacUsersRoute() {
  const { can } = useSession();
  return (
    <RequirePerm perm="users:read">
      <UsersPage canWriteUsers={can("users:write")} canReadRoles={can("roles:read")} />
    </RequirePerm>
  );
}

function RbacRolesRoute() {
  const { can } = useSession();
  return (
    <RequirePerm perm="roles:read">
      <RolesPage canWriteRoles={can("roles:write")} />
    </RequirePerm>
  );
}

function RbacPermissionsRoute() {
  const { can } = useSession();
  return (
    <RequirePerm perm="permissions:read">
      <PermissionsPage canWritePermissions={can("permissions:write")} />
    </RequirePerm>
  );
}

function SystemLayout() {
  return <Outlet />;
}

function SystemSettingsRoute() {
  const { can } = useSession();
  return <SystemPage canWriteSystem={can("system:write")} />;
}

function ReleasesRoute() {
  const { can } = useSession();
  return (
    <RequirePerm perm="releases:read">
      <ReleasesPage canWriteReleases={can("releases:write")} />
    </RequirePerm>
  );
}

function MapsRoute() {
  const { can } = useSession();
  return (
    <RequirePerm perm="maps:read">
      <MapsPage canWriteMaps={can("maps:write")} />
    </RequirePerm>
  );
}

function StorageRoute() {
  return (
    <RequirePerm perm="storage:read">
      <StoragePage />
    </RequirePerm>
  );
}

function SupportRoute() {
  const { user, roles, can } = useSession();
  return (
    <RequirePerm perm="support:read">
      <SupportPage
        userId={user?.id ?? ""}
        isAdmin={roles.includes("admin")}
        canWriteSupport={can("support:write")}
      />
    </RequirePerm>
  );
}

function FleetRoute() {
  return (
    <RequirePerm perm="fleet:read">
      <FleetPage />
    </RequirePerm>
  );
}

function OrgDetailWrapper() {
  const { id } = useParams();
  return <OrganizationDetailPage id={id ?? ""} />;
}

function Layout() {
  const { user, logout, can } = useSession();
  const location = useLocation();
  const [collapsed, setCollapsed] = useState<boolean>(() => {
    try {
      return localStorage.getItem(SIDEBAR_KEY) === "1";
    } catch {
      return false;
    }
  });

  function toggleSidebar() {
    setCollapsed((c) => {
      try {
        localStorage.setItem(SIDEBAR_KEY, c ? "0" : "1");
      } catch {
        /* private mode */
      }
      return !c;
    });
  }

  const filteredGroups = NAV.map((group) => ({
    ...group,
    items: group.items.filter((item) => !item.perm || can(item.perm)),
  })).filter((group) => group.items.length > 0);

  const isFullBleed =
    location.pathname.startsWith("/fleet") ||
    location.pathname.startsWith("/maps");

  return (
    <div className="min-h-dvh">
      <TitleBar
        collapsed={collapsed}
        onToggleSidebar={toggleSidebar}
        userName={user?.display_name || user?.email}
      />
      <Sidebar groups={filteredGroups} collapsed={collapsed} onLogout={logout} />
      <div
        className="gw-main-wrap"
        style={{
          paddingTop: 32,
          marginLeft: collapsed ? 60 : 188,
          height: isFullBleed ? "calc(100vh - 32px)" : "auto",
          transition: "margin-left 0.2s cubic-bezier(0.4, 0, 0.2, 1)",
        } as React.CSSProperties}
      >
        <main
          className={cn(
            isFullBleed ? "h-full w-full overflow-hidden p-0" : "p-4 md:p-6 lg:p-8"
          )}
        >
          <Outlet />
        </main>
      </div>
    </div>
  );
}
