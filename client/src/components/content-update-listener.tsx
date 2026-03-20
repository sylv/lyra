import { useApolloClient, useSubscription } from "@apollo/client/react";
import { graphql } from "../@generated/gql";
import { useSetup } from "./settings/setup/setup-wrapper";
import { isSetupReady } from "./settings/setup/setup-state";

const ContentUpdatesSubscription = graphql(`
	subscription ContentUpdates {
		contentUpdates
	}
`);

export const ContentUpdateListener = () => {
	const client = useApolloClient();
	const { refresh, state } = useSetup();
	const shouldSubscribe = state != null && isSetupReady(state);

	useSubscription(ContentUpdatesSubscription, {
		skip: !shouldSubscribe,
		onData: () => {
			void client.refetchQueries({ include: "active" });
			void refresh().catch(() => {});
		},
	});

	return null;
};
