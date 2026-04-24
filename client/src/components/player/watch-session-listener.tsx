import { useEffect } from "react";
import { useLocation, useNavigate } from "react-router";
import { useClient } from "urql";
import { graphql } from "../../@generated/gql";
import { useSetup } from "../settings/setup/setup-wrapper";
import { isSetupReady } from "../settings/setup/setup-state";
import { openPlayerMedia, setPendingWatchSession, usePlayerRuntimeStore } from "./player-runtime-store";

const GetWatchSession = graphql(`
  query GetWatchSession($sessionId: String!) {
    watchSession(sessionId: $sessionId) {
      id
      nodeId
    }
  }
`);

export const WatchSessionListener = () => {
  const client = useClient();
  const { state: setupState } = useSetup();
  const location = useLocation();
  const navigate = useNavigate();
  const { pathname, search: searchStr, hash } = location;
  const pendingSessionId = usePlayerRuntimeStore((state) => state.pendingWatchSessionId);

  useEffect(() => {
    if (!setupState || !isSetupReady(setupState)) return;

    const params = new URLSearchParams(searchStr);
    const linkSessionId = params.get("watchSession")?.trim() ?? "";
    if (!linkSessionId || linkSessionId === pendingSessionId) return;

    let cancelled = false;
    void client
      .query(GetWatchSession, { sessionId: linkSessionId }, { requestPolicy: "network-only" })
      .toPromise()
      .then(({ data }) => {
        const session = data?.watchSession ?? null;
        if (!session || cancelled) return;
        setPendingWatchSession(session.id, session.nodeId);
        openPlayerMedia(session.nodeId, false);

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
      .catch((watchSessionError) => {
        console.error("failed to load watch session from url", watchSessionError);
      });

    return () => {
      cancelled = true;
    };
  }, [client, hash, navigate, pathname, pendingSessionId, searchStr, setupState]);

  return null;
};
