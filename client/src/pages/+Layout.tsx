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
						<Sidebar>
							{children}
							<Player />
						</Sidebar>
						<Toaster />
					</Suspense>
					<DynamicBackground />
				</ApolloProvider>
			</TooltipProvider>
		</ThemeProvider>
	);
};
