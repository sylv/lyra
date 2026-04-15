import { Link } from "react-router";
import { graphql } from "../@generated/gql";
import { useSuspenseQuery } from "../hooks/use-suspense-query";
import { useTitle } from "../hooks/use-title";
import { getPathForCollection } from "../lib/getPathForMedia";

const CollectionsQuery = graphql(`
  query CollectionsIndex {
    collections {
      id
      name
      description
      itemCount
      visibility
      createdBy {
        username
      }
    }
  }
`);

export function CollectionsRoute() {
  useTitle("Collections");
  const [{ data }] = useSuspenseQuery({ query: CollectionsQuery });

  return (
    <div className="space-y-4 py-6">
      <div>
        <h1 className="text-2xl font-semibold">Collections</h1>
      </div>
      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        {data.collections.map((collection) => (
          <Link
            key={collection.id}
            to={getPathForCollection(collection.id)}
            className="rounded-lg border border-zinc-800 p-4 group"
          >
            <div className="flex items-start justify-between gap-4">
              <div>
                <div className="font-semibold group-hover:underline">{collection.name}</div>
                <div className="text-[11px] uppercase font-semibold text-zinc-500">
                  {collection.visibility.toLowerCase()}
                  {collection.createdBy ? ` by ${collection.createdBy.username}` : " system"}
                </div>
              </div>
              <div className="text-sm text-zinc-400">
                {collection.itemCount} item{collection.itemCount !== 1 ? "s" : ""}
              </div>
            </div>
            {collection.description ? (
              <p className="mt-3 line-clamp-3 text-sm text-zinc-300">{collection.description}</p>
            ) : null}
          </Link>
        ))}
      </div>
    </div>
  );
}
