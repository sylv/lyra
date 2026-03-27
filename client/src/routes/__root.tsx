import { ApolloProvider } from "@apollo/client/react";
import { TooltipProvider } from "@radix-ui/react-tooltip";
import { createRootRoute, Outlet, useLocation } from "@tanstack/react-router";
import { useStore } from "zustand";
import { apolloClientStore } from "../client";
import { DynamicBackground } from "../components/dynamic-background";
import { AppErrorBoundary } from "../components/error-boundary";
import { SuspenseBoundary } from "../components/fallback";
import { ContentUpdateListener } from "../components/content-update-listener";
import { PlayerWrapper } from "../components/player/player-wrapper";
import { WatchSessionListener } from "../components/player/watch-session-listener";
import { SetupWrapper } from "../components/settings/setup/setup-wrapper";
import { Sidebar } from "../components/sidebar";
import { Toaster } from "../components/ui/sonner";

export const Route = createRootRoute({
	component: RootComponent,
});

function RootComponent() {
	const client = useStore(apolloClientStore, (state) => state.client);
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const isSetupRoute = pathname === "/setup" || pathname.startsWith("/setup/");

	return (
		<TooltipProvider>
			<ApolloProvider client={client}>
				<AppErrorBoundary className="fixed inset-0">
					<SetupWrapper>
						<ContentUpdateListener />
						<WatchSessionListener />
						<AppErrorBoundary className="fixed inset-0">
							<SuspenseBoundary className="fixed inset-0">
								{isSetupRoute ? (
									<Outlet />
								) : (
									<>
										<Sidebar>
											<Outlet />
										</Sidebar>
										<PlayerWrapper />
									</>
								)}
							</SuspenseBoundary>
						</AppErrorBoundary>
					</SetupWrapper>
				</AppErrorBoundary>
				<Toaster />
				<DynamicBackground />
			</ApolloProvider>
		</TooltipProvider>
	);
}
