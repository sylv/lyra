import { useCallback, useEffect, useRef, useState } from "react";
import { useSubscription } from "urql";
import { graphql } from "../@generated/gql";
import { refreshActiveQueries } from "../client";
import { isSetupReady } from "./settings/setup/setup-state";
import { useSetup } from "./settings/setup/setup-wrapper";

const ContentUpdatesSubscription = graphql(`
	subscription ContentUpdates {
		contentUpdates
	}
`);

export const ContentUpdateListener = () => {
	const { recheckSetup, state } = useSetup();
	const [focused, setFocused] = useState(document.hasFocus());
	const unfocusedAt = useRef<number | null>(null);
	const shouldSubscribe = state != null && isSetupReady(state) && focused;

	const refreshAll = useCallback(async () => {
		await refreshActiveQueries();
		await recheckSetup().catch(() => {});
	}, [recheckSetup]);

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

	const [subscriptionResult] = useSubscription({
		query: ContentUpdatesSubscription,
		pause: !shouldSubscribe,
	});

	useEffect(() => {
		if (!subscriptionResult.data?.contentUpdates) {
			return;
		}

		void refreshAll();
	}, [refreshAll, subscriptionResult.data?.contentUpdates]);

	return null;
};
