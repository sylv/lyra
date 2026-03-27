import { useApolloClient, useSubscription } from "@apollo/client/react";
import { graphql } from "../@generated/gql";
import { useSetup } from "./settings/setup/setup-wrapper";
import { isSetupReady } from "./settings/setup/setup-state";
import { useCallback, useEffect, useRef, useState } from "react";

const ContentUpdatesSubscription = graphql(`
	subscription ContentUpdates {
		contentUpdates
	}
`);

export const ContentUpdateListener = () => {
	const client = useApolloClient();
	const { refresh, state } = useSetup();
	const [focused, setFocused] = useState(document.hasFocus());
	const unfocusedAt = useRef<number | null>(null);
	const shouldSubscribe = state != null && isSetupReady(state) && focused;

	const refreshAll = useCallback(async () => {
		await client.refetchQueries({ include: "active" });
		await refresh().catch(() => {});
	}, [client, refresh]);

	useEffect(() => {
		const handleChange = (focused: boolean) => {
			if (!focused) {
				unfocusedAt.current = Date.now();
			} else if (unfocusedAt.current) {
				const unfocusedDuration = Date.now() - unfocusedAt.current;
				if (unfocusedDuration > 10_000) {
					void refreshAll();
				}

				unfocusedAt.current = null;
			}

			setFocused(focused);
		};

		const handleFocus = () => handleChange(true);
		const handleBlur = () => handleChange(false);
		window.addEventListener("focus", handleFocus);
		window.addEventListener("blur", handleBlur);

		return () => {
			window.removeEventListener("focus", handleFocus);
			window.removeEventListener("blur", handleBlur);
		};
	}, [refreshAll]);

	useSubscription(ContentUpdatesSubscription, {
		skip: !shouldSubscribe,
		onData: () => {
			void refreshAll();
		},
	});

	return null;
};
