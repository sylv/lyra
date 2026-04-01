import { useEffect } from "react";
import { useLocation, useNavigate } from "react-router";
import { useClient } from "urql";
import { unmask } from "../../@generated/gql";
import { useSetup } from "../settings/setup/setup-wrapper";
import { isSetupReady } from "../settings/setup/setup-state";
import { setPlayerMedia, setPlayerState, usePlayerContext } from "./player-context";
import { GetWatchSession, WatchSessionSummary } from "./player-queries";
import { setPendingWatchSession } from "./watch-session";

export const WatchSessionListener = () => {
	const client = useClient();
	const { state: setupState } = useSetup();
	const location = useLocation();
	const navigate = useNavigate();
	const { pathname, search: searchStr, hash } = location;
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
			.query(GetWatchSession, { sessionId: linkSessionId }, { requestPolicy: "network-only" })
			.toPromise()
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
				navigate(
					{
						pathname,
						search: nextSearch ? `?${nextSearch}` : "",
						hash,
					},
					{ replace: true },
				);
			})
			.catch((error) => {
				console.error("failed to load watch session from url", error);
			});

		return () => {
			cancelled = true;
		};
	}, [client, hash, navigate, pathname, pendingSessionId, searchStr, sessionId, setupState]);

	return null;
};
