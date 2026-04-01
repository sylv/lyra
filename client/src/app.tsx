import { TooltipProvider } from "@radix-ui/react-tooltip";
import type { FC } from "react";
import { useLocation } from "react-router";
import { Provider as UrqlProvider } from "urql";
import { client } from "./client";
import { ContentUpdateListener } from "./components/content-update-listener";
import { DynamicBackground } from "./components/dynamic-background";
import { AppErrorBoundary } from "./components/error-boundary";
import { PlayerWrapper } from "./components/player/player-wrapper";
import { WatchSessionListener } from "./components/player/watch-session-listener";
import { SetupWrapper } from "./components/settings/setup/setup-wrapper";
import { Sidebar } from "./components/sidebar";
import { Toaster } from "./components/ui/sonner";
import { AppRoutes } from "./routes";

export const App: FC = () => {
	return (
		<UrqlProvider value={client}>
			<TooltipProvider>
				<AppErrorBoundary className="fixed inset-0">
					<SetupWrapper>
						<ContentUpdateListener />
						<WatchSessionListener />
						<AppErrorBoundary className="fixed inset-0">
							<LayoutWrapper />
						</AppErrorBoundary>
					</SetupWrapper>
				</AppErrorBoundary>
				<Toaster />
				<DynamicBackground />
			</TooltipProvider>
		</UrqlProvider>
	);
};

export const LayoutWrapper: FC = () => {
	const location = useLocation();
	const isSetupRoute = location.pathname.startsWith("/setup");

	if (isSetupRoute) return <AppRoutes />;
	return (
		<>
			<Sidebar>
				<AppRoutes />
			</Sidebar>
			<PlayerWrapper />
		</>
	);
};
