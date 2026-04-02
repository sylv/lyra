import { Link } from "react-router";
import { useQuery } from "urql";
import { graphql } from "../@generated/gql";
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
	const [{ data }] = useQuery({ query: CollectionsQuery, context: { suspense: true } });

	return (
		<div className="space-y-4 py-6">
			<div>
				<h1 className="text-2xl font-semibold">Collections</h1>
				<p className="mt-1 text-sm text-zinc-400">Browse your private shelves and the shared ones worth keeping around.</p>
			</div>
			<div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
				{data?.collections.map((collection) => (
					<Link
						key={collection.id}
						to={getPathForCollection(collection.id)}
						className="rounded-lg border border-zinc-800 bg-black/20 p-4 transition hover:bg-zinc-950/70"
					>
						<div className="flex items-start justify-between gap-4">
							<div>
								<div className="font-semibold hover:underline">{collection.name}</div>
								<div className="mt-1 text-xs uppercase tracking-wide text-zinc-500">
									{collection.visibility.toLowerCase()}
									{collection.createdBy ? ` by ${collection.createdBy.username}` : " system"}
								</div>
							</div>
							<div className="text-sm text-zinc-400">{collection.itemCount}</div>
						</div>
						{collection.description ? <p className="mt-3 line-clamp-3 text-sm text-zinc-300">{collection.description}</p> : null}
					</Link>
				))}
			</div>
		</div>
	);
}
