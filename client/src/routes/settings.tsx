import { type FC } from "react";
import { Navigate, Outlet, useLocation, useNavigate } from "react-router";
import { useQuery } from "urql";
import { graphql } from "../@generated/gql";
import { Tabs, TabsList, TabsTrigger } from "../components/ui/tabs";
import { useTitle } from "../hooks/use-title";
import { ADMIN_BIT } from "../lib/user-permissions";

export const settingsTabs = {
	users: "/settings/users",
	sessions: "/settings/sessions",
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
export const SettingsRoute: FC = () => {
	const location = useLocation();
	const navigate = useNavigate();
	const pathname = location.pathname;
	const [{ data }] = useQuery({ query: SettingsViewerQuery, context: { suspense: true } });
	const viewerPermissions = data?.viewer?.permissions ?? 0;
	const canManageUsers = (viewerPermissions & ADMIN_BIT) !== 0;
	const canViewSessions = (viewerPermissions & ADMIN_BIT) !== 0;
	const canManageLibraries = (viewerPermissions & ADMIN_BIT) !== 0;
	const visibleTabs = [
		canManageUsers ? "users" : null,
		canViewSessions ? "sessions" : null,
		canManageLibraries ? "libraries" : null,
		"about",
		"import",
	].filter((tab): tab is SettingsTab => tab != null);
	const fallbackTab = visibleTabs[0] ?? "about";
	const activeTab: SettingsTab = pathname.startsWith(settingsTabs.users)
		? "users"
		: pathname.startsWith(settingsTabs.sessions)
			? "sessions"
			: pathname.startsWith(settingsTabs.libraries)
				? "libraries"
				: pathname.startsWith(settingsTabs.import)
					? "import"
					: "about";
	const activeTabVisible =
		activeTab === "users"
			? canManageUsers
			: activeTab === "sessions"
				? canViewSessions
				: activeTab === "libraries"
					? canManageLibraries
					: true;

	useTitle("Settings");

	if (!activeTabVisible) {
		return <Navigate to={settingsTabs[fallbackTab]} replace />;
	}

	return (
		<div className="pt-6">
			<Tabs
				className="w-full"
				value={activeTab}
				onValueChange={(value) => {
					const nextPath = settingsTabs[value as SettingsTab];
					if (!nextPath) {
						return;
					}

					navigate(nextPath);
				}}
			>
				<TabsList>
					{canManageUsers ? <TabsTrigger value="users">Users</TabsTrigger> : null}
					{canViewSessions ? <TabsTrigger value="sessions">Sessions</TabsTrigger> : null}
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
};
