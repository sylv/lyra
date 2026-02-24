import { useApolloClient, useQuery } from "@apollo/client/react";
import { graphql } from "gql.tada";
import { type FC, useEffect, useRef } from "react";

const Query = graphql(`
	query SyncVersion {
		syncVersion
	}
`);

export const SyncVersionWatcher: FC = () => {
	const client = useApolloClient();
	const initializedVersionRef = useRef<number | null>(null);
	const { data } = useQuery(Query, {
		fetchPolicy: "network-only",
		nextFetchPolicy: "network-only",
		pollInterval: 4000,
	});

	useEffect(() => {
		const nextVersion = data?.syncVersion;
		if (nextVersion == null) {
			return;
		}

		if (initializedVersionRef.current == null) {
			initializedVersionRef.current = nextVersion;
			return;
		}

		if (initializedVersionRef.current === nextVersion) {
			return;
		}

		initializedVersionRef.current = nextVersion;
		void client.refetchQueries({ include: "active" });
	}, [client, data?.syncVersion]);

	return null;
};
