import { useApolloClient } from "@apollo/client/react";
import { useLocation } from "@tanstack/react-router";
import { useEffect } from "react";
import { unmask } from "../../@generated/gql";
import { useSetup } from "../settings/setup/setup-wrapper";
import { isSetupReady } from "../settings/setup/setup-state";
import { setPlayerMedia, setPlayerState, usePlayerContext } from "./player-context";
import { GetWatchSession, WatchSessionSummary } from "./player-queries";
import { setPendingWatchSession } from "./watch-session";

export const WatchSessionListener = () => {
	const client = useApolloClient();
	const { state: setupState } = useSetup();
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const searchStr = useLocation({
		select: (location) => location.searchStr,
	});
	const hash = useLocation({
		select: (location) => location.hash,
	});
	const sessionId = usePlayerContext((ctx) => ctx.watchSession.sessionId);
	const pendingSessionId = usePlayerContext((ctx) => ctx.watchSession.pendingSessionId);

	useEffect(() => {
		if (!setupState || !isSetupReady(setupState)) return;

		const params = new URLSearchParams(searchStr);
		const linkSessionId = params.get("watchSession")?.trim() ?? "";
		if (!linkSessionId) return;
		if (linkSessionId === sessionId || linkSessionId === pendingSessionId) return;

		let cancelled = false;
		void client
			.query({
				query: GetWatchSession,
				variables: { sessionId: linkSessionId },
				fetchPolicy: "network-only",
			})
			.then(({ data }) => {
				const session = data?.watchSession ? unmask(WatchSessionSummary, data.watchSession) : null;
				if (!session || cancelled) return;
				setPendingWatchSession(session.id, session.nodeId);
				setPlayerMedia(session.nodeId, false);
				setPlayerState({
					isFullscreen: true,
					shouldPromptResume: false,
					pendingInitialPosition: null,
				});

				params.delete("watchSession");
				const nextSearch = params.toString();
				const nextUrl = `${pathname}${nextSearch ? `?${nextSearch}` : ""}${hash}`;
				window.history.replaceState({}, "", nextUrl);
			})
			.catch((error) => {
				console.error("failed to load watch session from url", error);
			});

		return () => {
			cancelled = true;
		};
	}, [client, hash, pathname, pendingSessionId, searchStr, sessionId, setupState]);

	return null;
};
