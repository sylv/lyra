import { useState } from "react";
import { graphql } from "../@generated/gql";
import { NodeKind, OrderBy } from "../@generated/gql/graphql";
import { NodePosterDetail } from "../components/nodes/node-poster-detail";
import { ShelfCarousel } from "../components/shelf-carousel";
import { Tabs, TabsList, TabsTrigger } from "../components/ui/tabs";
import { useSuspenseQuery } from "../hooks/use-suspense-query";
import { useTitle } from "../hooks/use-title";

const RecentlyAddedQuery = graphql(`
  query HomeRecentlyAdded {
    nodeList(first: 24, filter: { kinds: [MOVIE, SERIES], orderBy: ADDED_AT }) {
      nodes {
        id
        ...NodePoster
      }
    }
  }
`);

const WatchedQuery = graphql(`
  query HomeWatched {
    nodeList(
      first: 24
      filter: { kinds: [MOVIE, SERIES, EPISODE], watched: true, orderBy: WATCH_PROGRESS_UPDATED_AT }
    ) {
      nodes {
        id
        ...NodePoster
      }
    }
  }
`);

function RecentlyAddedTab() {
  const [{ data }] = useSuspenseQuery({ query: RecentlyAddedQuery });
  const nodes = data.nodeList.nodes;

  if (nodes.length === 0) {
    return <p className="text-sm text-zinc-400 py-4">No media found. Add a library to get started.</p>;
  }

  return (
    <ShelfCarousel title={<span className="text-lg font-semibold">Recently Added</span>}>
      {nodes.map((node) => (
        <NodePosterDetail key={node.id} node={node} className="min-w-0 flex-[0_0_8.25rem]" />
      ))}
    </ShelfCarousel>
  );
}

function WatchedTab() {
  const [{ data }] = useSuspenseQuery({ query: WatchedQuery });
  const nodes = data.nodeList.nodes;

  if (nodes.length === 0) {
    return <p className="text-sm text-zinc-400 py-4">Nothing watched yet.</p>;
  }

  return (
    <ShelfCarousel title={<span className="text-lg font-semibold">Watched</span>}>
      {nodes.map((node) => (
        <NodePosterDetail key={node.id} node={node} className="min-w-0 flex-[0_0_8.25rem]" />
      ))}
    </ShelfCarousel>
  );
}

type HomeTab = "recently-added" | "watched";

export function HomeRoute() {
  useTitle("Home");
  const [activeTab, setActiveTab] = useState<HomeTab>("recently-added");

  return (
    <div className="py-6 space-y-4">
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as HomeTab)}>
        <TabsList>
          <TabsTrigger value="recently-added">Recently Added</TabsTrigger>
          <TabsTrigger value="watched">Watched</TabsTrigger>
        </TabsList>
      </Tabs>
      <div>
        {activeTab === "recently-added" && <RecentlyAddedTab />}
        {activeTab === "watched" && <WatchedTab />}
      </div>
    </div>
  );
}
