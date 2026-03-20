import { useSuspenseQuery } from "@apollo/client/react";
import { Navigate, Outlet, createFileRoute, useLocation, useNavigate } from "@tanstack/react-router";
import { graphql } from "../@generated/gql";
import { Tabs, TabsList, TabsTrigger } from "../components/ui/tabs";
import { useTitle } from "../hooks/use-title";
import { ADMIN_BIT } from "../lib/user-permissions";

const settingsTabs = {
	users: "/settings/users",
	libraries: "/settings/libraries",
	about: "/settings/about",
	import: "/settings/import",
} as const;

type SettingsTab = keyof typeof settingsTabs;

const SettingsViewerQuery = graphql(`
	query SettingsViewer {
		viewer {
			id
			permissions
		}
	}
`);

export const Route = createFileRoute("/settings")({
	component: RouteComponent,
});

function RouteComponent() {
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const navigate = useNavigate();
	const { data } = useSuspenseQuery(SettingsViewerQuery);
	const viewerPermissions = data.viewer?.permissions ?? 0;
	const canManageUsers = (viewerPermissions & ADMIN_BIT) !== 0;
	const canManageLibraries = (viewerPermissions & ADMIN_BIT) !== 0;
	const visibleTabs = [
		canManageUsers ? "users" : null,
		canManageLibraries ? "libraries" : null,
		"about",
		"import",
	].filter((tab): tab is SettingsTab => tab != null);
	const fallbackTab = visibleTabs[0] ?? "about";
	const activeTab: SettingsTab = pathname.startsWith(settingsTabs.users)
		? "users"
		: pathname.startsWith(settingsTabs.libraries)
			? "libraries"
			: pathname.startsWith(settingsTabs.import)
				? "import"
				: "about";
	const activeTabVisible =
		activeTab === "users" ? canManageUsers : activeTab === "libraries" ? canManageLibraries : true;

	useTitle("Settings");

	if (!activeTabVisible) {
		return <Navigate to={settingsTabs[fallbackTab]} replace />;
	}

	return (
		<div className="pt-6">
			<Tabs
				value={activeTab}
				onValueChange={(value) => {
					const nextPath = settingsTabs[value as SettingsTab];
					if (!nextPath) {
						return;
					}

					void navigate({ to: nextPath });
				}}
				className="w-full"
			>
				<TabsList>
					{canManageUsers ? <TabsTrigger value="users">Users</TabsTrigger> : null}
					{canManageLibraries ? <TabsTrigger value="libraries">Libraries</TabsTrigger> : null}
					<TabsTrigger value="about">About</TabsTrigger>
					<TabsTrigger value="import">Import</TabsTrigger>
				</TabsList>
				<div className="min-h-[70vh] rounded-xl border border-zinc-700/60 bg-zinc-500/20 p-6">
					<Outlet />
				</div>
			</Tabs>
		</div>
	);
}
