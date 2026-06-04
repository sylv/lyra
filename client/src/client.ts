import { cacheExchange } from "@urql/exchange-graphcache";
import { retryExchange } from "@urql/exchange-retry";
import { createClient as createUrqlClient, fetchExchange, type Exchange, type Operation } from "urql";
import { pipe, subscribe, tap } from "wonka";
import { cacheUpdates } from "./cache";

const activeQueries = new Map<number, Operation>();
const refreshExchange: Exchange = ({ forward }) => {
  return (ops$) => {
    return forward(
      pipe(
        ops$,
        tap((op) => {
          if (op.kind === "query") {
            activeQueries.set(op.key, op);
          } else if (op.kind === "teardown") {
            activeQueries.delete(op.key);
          }
        }),
      ),
    );
  };
};

export const client = createUrqlClient({
  url: "/api/graphql",
  preferGetMethod: false,
  exchanges: [
    cacheExchange({
      keys: {
        NodeProperties: () => null,
        FileProbe: () => null,
        PlaybackOptions: () => null,
        VideoRenditionOption: () => null,
        AudioTrackOption: () => null,
        AudioRenditionOption: () => null,
        SubtitlePlaybackTrack: () => null,
        SubtitleRenditionOption: () => null,
        CastMember: () => null,
        ContentRating: () => null,
        MetadataGenre: () => null,
        Activity: (data) => data.taskType as any,
      },
      updates: cacheUpdates,
    }),
    refreshExchange,
    retryExchange({
      initialDelayMs: 1000,
      maxDelayMs: 30 * 1000,
      maxNumberAttempts: 10,
    }),
    fetchExchange,
  ],
});

export const refreshActiveQueries = (): Promise<void> => {
  if (activeQueries.size === 0) return Promise.resolve();
  const promises = Array.from(activeQueries.values()).map(
    (op) =>
      new Promise<void>((resolve, reject) => {
        const { unsubscribe } = pipe(
          client.executeQuery(op, { requestPolicy: "cache-and-network" }),
          subscribe((result) => {
            // stale: true = cache-and-network emitted a cache hit while the
            // network request is still in flight; wait for the real response.
            if (result.stale) return;
            unsubscribe();
            if (result.error) reject(result.error);
            else resolve();
          }),
        );
      }),
  );

  return Promise.all(promises).then(() => void 0);
};
