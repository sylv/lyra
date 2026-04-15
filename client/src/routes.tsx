import { lazy, type FC } from "react";
import { Navigate, Route, Routes } from "react-router";

const HomeRoute = lazy(() => import("./routes/home").then((module) => ({ default: module.HomeRoute })));
const CollectionRoute = lazy(() =>
  import("./routes/collection").then((module) => ({ default: module.CollectionRoute })),
);
const CollectionsRoute = lazy(() =>
  import("./routes/collections").then((module) => ({ default: module.CollectionsRoute })),
);
const LibraryNodeRoute = lazy(() =>
  import("./routes/library_node").then((module) => ({ default: module.LibraryNodeRoute })),
);
const LibraryRoute = lazy(() => import("./routes/library").then((module) => ({ default: module.LibraryRoute })));
const PlaygroundRoute = lazy(() =>
  import("./routes/playground").then((module) => ({ default: module.PlaygroundRoute })),
);
const SettingsRoute = lazy(() => import("./routes/settings").then((module) => ({ default: module.SettingsRoute })));
const SettingsAboutRoute = lazy(() =>
  import("./routes/settings_about").then((module) => ({ default: module.SettingsAboutRoute })),
);
const SettingsImportRoute = lazy(() =>
  import("./routes/settings_import").then((module) => ({ default: module.SettingsImportRoute })),
);
const SettingsLibrariesRoute = lazy(() =>
  import("./routes/settings_libraries").then((module) => ({ default: module.SettingsLibrariesRoute })),
);
const SettingsSessionsRoute = lazy(() =>
  import("./routes/settings_sessions").then((module) => ({ default: module.SettingsSessionsRoute })),
);
const SettingsUsersRoute = lazy(() =>
  import("./routes/settings_users").then((module) => ({ default: module.SettingsUsersRoute })),
);
const SetupCreateAccountRoute = lazy(() =>
  import("./routes/setup_create_account").then((module) => ({ default: module.SetupCreateAccountRoute })),
);
const SetupCreateLibraryRoute = lazy(() =>
  import("./routes/setup_create_library").then((module) => ({ default: module.SetupCreateLibraryRoute })),
);
const SetupLoginRoute = lazy(() =>
  import("./routes/setup_login").then((module) => ({ default: module.SetupLoginRoute })),
);
const SetupRoute = lazy(() => import("./routes/setup").then((module) => ({ default: module.SetupRoute })));

export const AppRoutes: FC = () => (
  <Routes>
    <Route path="/setup" element={<SetupRoute />}>
      <Route path="login" element={<SetupLoginRoute />} />
      <Route path="create-account" element={<SetupCreateAccountRoute />} />
      <Route path="create-library" element={<SetupCreateLibraryRoute />} />
    </Route>
    <Route path="/playground" element={<PlaygroundRoute />} />
    <Route path="/collection/:collectionId" element={<CollectionRoute />} />
    <Route path="/collections" element={<CollectionsRoute />} />
    <Route path="/library/:libraryId/node/:nodeId" element={<LibraryNodeRoute />} />
    <Route path="/library/:libraryId" element={<LibraryRoute />} />
    <Route path="/settings" element={<SettingsRoute />}>
      <Route path="users" element={<SettingsUsersRoute />} />
      <Route path="sessions" element={<SettingsSessionsRoute />} />
      <Route path="libraries" element={<SettingsLibrariesRoute />} />
      <Route path="import" element={<SettingsImportRoute />} />
      <Route path="about" element={<SettingsAboutRoute />} />
      <Route index element={<Navigate to="/settings/about" replace />} />
    </Route>
    <Route path="/" element={<HomeRoute />} />
    <Route path="*" element={<Navigate to="/" replace />} />
  </Routes>
);
