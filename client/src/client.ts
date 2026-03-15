import { ApolloClient, InMemoryCache } from "@apollo/client";
import { HttpLink } from "@apollo/client/link/http";
import { relayStylePagination } from "@apollo/client/utilities";
import { create } from "zustand";

const createApolloClient = () =>
	new ApolloClient({
		link: new HttpLink({
			uri: "/api/graphql",
		}),
		cache: new InMemoryCache({
			typePolicies: {
				Query: {
					fields: {
						nodeList: relayStylePagination(["filter"]),
					},
				},
			},
		}),
	});

interface ApolloClientState {
	client: ApolloClient;
	resetClient: () => ApolloClient;
}

export const apolloClientStore = create<ApolloClientState>()((set, get) => ({
	client: createApolloClient(),
	resetClient: () => {
		get().client.stop();
		const client = createApolloClient();
		set({ client });
		return client;
	},
}));

export const getApolloClient = () => apolloClientStore.getState().client;

export const resetApolloClient = () => apolloClientStore.getState().resetClient();
