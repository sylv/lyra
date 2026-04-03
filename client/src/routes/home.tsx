import { useQuery } from "urql";
import { graphql } from "../@generated/gql";
import { CollectionShelf } from "../components/collection-shelf";
import { NodeList } from "../components/nodes/node-list";
import { useTitle } from "../hooks/use-title";

const HomeQuery = graphql(`
	query HomeCollections {
		home {
			sections {
				id
				...CollectionShelf
			}
		}
	}
`);

export function HomeRoute() {
	useTitle("Home");
	const [{ data }] = useQuery({ query: HomeQuery, context: { suspense: true } });

	return (
		<div className="space-y-8 py-6">
			{data?.home.sections.map((section) => (
				<CollectionShelf key={section.id} collection={section} />
			))}
			<section>
				<h2 className="text-xl font-semibold mb-2">Everything else</h2>
				<NodeList type="movies_series" />
			</section>
		</div>
	);
}
