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
						rootList: relayStylePagination(["filter"]),
						itemList: relayStylePagination(["filter"]),
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
	// We keep the Apollo client in zustand so auth resets can swap the provider's client via React state.
	// `refetchQueries` and `clearStore` were the obvious fixes, but they still left a stale 401 stuck in
	// suspense/error-boundary recovery. Recreating the client was the only reliable way we found to drop
	// that state after logging in from a protected route.
	resetClient: () => {
		get().client.stop();
		const client = createApolloClient();
		set({ client });
		return client;
	},
}));

export const getApolloClient = () => apolloClientStore.getState().client;

export const resetApolloClient = () => apolloClientStore.getState().resetClient();
