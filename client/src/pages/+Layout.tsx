import { ApolloClient, InMemoryCache } from "@apollo/client";
import { HttpLink } from "@apollo/client/link/http";
import { ApolloProvider } from "@apollo/client/react";
import { relayStylePagination } from "@apollo/client/utilities";
import { TooltipProvider } from "@radix-ui/react-tooltip";
import type { FC, ReactNode } from "react";
import { DynamicBackground } from "../components/dynamic-background";
import { SuspenseBoundary } from "../components/fallback";
import { PlayerWrapper } from "../components/player/player-wrapper";
import { SetupWrapper } from "../components/setup/setup-wrapper";
import { Sidebar } from "../components/sidebar";
import { ThemeProvider } from "../components/theme-provider";
import { Toaster } from "../components/ui/sonner";
import "./globals.css";

const client = new ApolloClient({
	link: new HttpLink({
		uri: "/api/graphql",
	}),
	cache: new InMemoryCache({
		typePolicies: {
			Query: {
				fields: {
					rootList: relayStylePagination(["filter"]),
					itemList: relayStylePagination(["filter"]),
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
					<SuspenseBoundary className="fixed inset-0">
						<SetupWrapper>
							<Sidebar>{children}</Sidebar>
							<PlayerWrapper />
						</SetupWrapper>
					</SuspenseBoundary>
					<Toaster />
					<div className="fixed inset-0 h-dvw w-dvw">
						<DynamicBackground />
					</div>
				</ApolloProvider>
			</TooltipProvider>
		</ThemeProvider>
	);
};
