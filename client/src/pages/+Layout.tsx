import { ApolloClient, ApolloProvider, InMemoryCache } from "@apollo/client";
import { relayStylePagination } from "@apollo/client/utilities";
import { TooltipProvider } from "@radix-ui/react-tooltip";
import type { FC, ReactNode } from "react";
import { Suspense } from "react";
import { DynamicBackground } from "../components/dynamic-background";
import { Fallback } from "../components/fallback";
import { PlayerWrapper } from "../components/player/player-wrapper";
import { SetupWrapper } from "../components/setup/setup-wrapper";
import { Sidebar } from "../components/sidebar";
import { ThemeProvider } from "../components/theme-provider";
import { Toaster } from "../components/ui/sonner";
import "./globals.css";
import { SearchModal } from "../components/search/search-modal";

const client = new ApolloClient({
	uri: "/api/graphql",
	cache: new InMemoryCache({
		typePolicies: {
			Query: {
				fields: {
					mediaList: relayStylePagination(["filter"]),
				},
			},
		},
	}),
});

export const Layout: FC<{ children: ReactNode }> = ({ children }) => {
	return (
		<ThemeProvider>
			<TooltipProvider>
				<ApolloProvider client={client}>
					<Suspense fallback={<Fallback />}>
						<SetupWrapper>
							<Sidebar>{children}</Sidebar>
							<PlayerWrapper />
							<SearchModal />
						</SetupWrapper>
					</Suspense>
					<Toaster />
					<div className="fixed inset-0 h-dvw w-dvw">
						<DynamicBackground />
					</div>
				</ApolloProvider>
			</TooltipProvider>
		</ThemeProvider>
	);
};
