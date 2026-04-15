import { graphql } from "../@generated/gql";
import { CollectionShelf } from "../components/collection-shelf";
import { useSuspenseQuery } from "../hooks/use-suspense-query";
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
	const [{ data }] = useSuspenseQuery({ query: HomeQuery });

	return (
		<div className="space-y-8 py-6">
			{data.home.sections.map((section) => (
				<CollectionShelf key={section.id} collection={section} />
			))}
		</div>
	);
}
