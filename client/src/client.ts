import { ApolloClient, InMemoryCache, split } from "@apollo/client";
import { HttpLink } from "@apollo/client/link/http";
import { GraphQLWsLink } from "@apollo/client/link/subscriptions";
import { relayStylePagination } from "@apollo/client/utilities";
import { OperationTypeNode } from "graphql";
import { createClient } from "graphql-ws";
import { create } from "zustand";

const getGraphqlWebsocketUrl = () => {
	const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
	return `${protocol}//${window.location.host}/api/graphql/ws`;
};

const createApolloClient = () => {
	const httpLink = new HttpLink({
		uri: "/api/graphql",
	});
	const wsLink = new GraphQLWsLink(
		createClient({
			url: getGraphqlWebsocketUrl(),
			retryAttempts: Number.MAX_SAFE_INTEGER,
		}),
	);
	const link = split(({ operationType }) => operationType === OperationTypeNode.SUBSCRIPTION, wsLink, httpLink);

	return new ApolloClient({
		link,
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
};

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
