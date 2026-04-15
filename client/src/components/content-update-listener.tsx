import { useCallback, useEffect } from "react";
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
  const shouldSubscribe = state != null && isSetupReady(state);

  const refreshAll = useCallback(async () => {
    await refreshActiveQueries();
    await recheckSetup().catch(() => {});
  }, [recheckSetup]);

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
