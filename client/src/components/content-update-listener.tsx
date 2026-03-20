import { useApolloClient, useSubscription } from "@apollo/client/react";
import { graphql } from "../@generated/gql";
import { useSetup } from "./settings/setup/setup-wrapper";

const ContentUpdatesSubscription = graphql(`
	subscription ContentUpdates {
		contentUpdates
	}
`);

export const ContentUpdateListener = () => {
	const client = useApolloClient();
	const { refresh } = useSetup();

	useSubscription(ContentUpdatesSubscription, {
		onData: () => {
			void client.refetchQueries({ include: "active" });
			void refresh().catch(() => {});
		},
	});

	return null;
};
