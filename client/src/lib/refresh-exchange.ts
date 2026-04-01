import type { Client, Exchange, Operation } from "@urql/core";
import { pipe, tap } from "wonka";

export function createBackgroundRefreshController() {
	const watched = new Map<number, Operation>();
	let clientRef: Client | null = null;

	const exchange: Exchange = ({ client, forward }) => {
		clientRef = client;

		return (ops$) => {
			return forward(
				pipe(
					ops$,
					tap((op) => {
						if (op.kind === "query") {
							watched.set(op.key, op);
						} else if (op.kind === "teardown") {
							watched.delete(op.key);
						}
					}),
				),
			);
		};
	};

	function refreshAll() {
		const client = clientRef;
		if (!client) return;

		for (const op of watched.values()) {
			client.reexecuteOperation(
				client.createRequestOperation("query", op, {
					...op.context,
					requestPolicy: "cache-and-network",
				}),
			);
		}
	}

	return { exchange, refreshAll };
}
