import { Outlet, createFileRoute, useLocation, useNavigate } from "@tanstack/react-router";
import { Tabs, TabsList, TabsTrigger } from "../components/ui/tabs";
import { useTitle } from "../hooks/use-title";

const settingsTabs = {
	about: "/settings/about",
	import: "/settings/import",
} as const;

type SettingsTab = keyof typeof settingsTabs;

export const Route = createFileRoute("/settings")({
	component: RouteComponent,
});

function RouteComponent() {
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const navigate = useNavigate();
	const activeTab: SettingsTab = pathname.startsWith(settingsTabs.import) ? "import" : "about";

	useTitle("Settings");

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
					<TabsTrigger value="about">About</TabsTrigger>
					<TabsTrigger value="import">Import</TabsTrigger>
				</TabsList>
				<div className="min-h-[70vh] rounded bg-zinc-400/10 p-6">
					<Outlet />
				</div>
			</Tabs>
		</div>
	);
}
