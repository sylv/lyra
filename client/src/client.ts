import { cacheExchange } from "@urql/exchange-graphcache";
import { retryExchange } from "@urql/exchange-retry";
import { createClient as createWsClient } from "graphql-ws";
import {
	createClient as createUrqlClient,
	fetchExchange,
	subscriptionExchange,
	type Exchange,
	type Operation,
} from "urql";
import { pipe, subscribe, tap } from "wonka";
import { cacheUpdates } from "./cache";

const getGraphqlWebsocketUrl = () => {
	const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
	return `${protocol}//${window.location.host}/api/graphql/ws`;
};

const wsClient = createWsClient({
	url: getGraphqlWebsocketUrl(),
	lazy: false,
	retryAttempts: Infinity,
	retryWait: (retries) => {
		const timeout = Math.min(1000 * 2 ** retries, 5 * 60 * 1000);
		return new Promise((resolve) => setTimeout(resolve, timeout));
	},
});

const activeQueries = new Map<number, Operation>();
const refreshExchange: Exchange = ({ forward }) => {
	return (ops$) => {
		return forward(
			pipe(
				ops$,
				tap((op) => {
					if (op.kind === "query") {
						activeQueries.set(op.key, op);
					} else if (op.kind === "teardown") {
						activeQueries.delete(op.key);
					}
				}),
			),
		);
	};
};

export const client = createUrqlClient({
	url: "/api/graphql",
	preferGetMethod: false,
	exchanges: [
		cacheExchange({
			keys: {
				NodeProperties: () => null,
				HomeView: () => null,
				Activity: (data) => data.taskType as any,
			},
			updates: cacheUpdates,
		}),

		subscriptionExchange({
			forwardSubscription: (request) => ({
				subscribe: (sink) => ({
					unsubscribe: wsClient.subscribe(
						{
							...request,
							query: request.query ?? "",
						},
						sink,
					),
				}),
			}),
		}),
		refreshExchange,
		retryExchange({
			initialDelayMs: 1000,
			maxDelayMs: 30 * 1000,
			maxNumberAttempts: 10,
		}),
		fetchExchange,
	],
});

export const refreshActiveQueries = (): Promise<void> => {
	if (activeQueries.size === 0) return Promise.resolve();
	const promises = Array.from(activeQueries.values()).map(
		(op) =>
			new Promise<void>((resolve, reject) => {
				const { unsubscribe } = pipe(
					client.executeQuery(op, { requestPolicy: "cache-and-network" }),
					subscribe((result) => {
						// stale: true = cache-and-network emitted a cache hit while the
						// network request is still in flight; wait for the real response.
						if (result.stale) return;
						unsubscribe();
						if (result.error) reject(result.error);
						else resolve();
					}),
				);
			}),
	);

	return Promise.all(promises).then(() => void 0);
};
