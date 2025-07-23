import { TooltipProvider } from "@radix-ui/react-tooltip";
import type { FC, ReactNode } from "react";
import { Suspense } from "react";
import { ApolloClient, ApolloProvider, InMemoryCache } from "@apollo/client";
import { DynamicBackground } from "../components/dynamic-background";
import { Fallback } from "../components/fallback";
import { Player } from "../components/player/player";
import { Sidebar } from "../components/sidebar";
import { ThemeProvider } from "../components/theme-provider";
import { Toaster } from "../components/ui/sonner";
import "./globals.css";
import { SetupWrapper } from "../components/setup/setup-wrapper";

const client = new ApolloClient({
	uri: "/api/graphql",
	cache: new InMemoryCache(),
});

export const Layout: FC<{ children: ReactNode }> = ({ children }) => {
	return (
		<ThemeProvider>
			<TooltipProvider>
				<ApolloProvider client={client}>
					<Suspense fallback={<Fallback />}>
						<SetupWrapper>
							<Sidebar>
								{children}
								<Player />
							</Sidebar>
						</SetupWrapper>
						<Toaster />
					</Suspense>
					<div className="fixed inset-0 h-dvw w-dvw">
						<DynamicBackground />
					</div>
				</ApolloProvider>
			</TooltipProvider>
		</ThemeProvider>
	);
};
