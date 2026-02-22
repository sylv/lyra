import { useSuspenseQuery } from "@apollo/client/react";
import { graphql, readFragment } from "gql.tada";
import { Fragment } from "react/jsx-runtime";
import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader, MediaHeaderFrag } from "../../../../../components/media-header";
import { SeasonCard, SeasonCardFrag } from "../../../../../components/season-card";

const Query = graphql(
	`
	query GetRootById($rootId: String!) {
		root(rootId: $rootId) {
			id
			kind
			libraryId
			seasons {
				id
				...SeasonCard
			}
			...MediaHeader
		}
	}
`,
	[MediaHeaderFrag, SeasonCardFrag],
);

export default function Page() {
	const pageContext = usePageContext();
	const rootId = pageContext.routeParams.rootId;
	const { data } = useSuspenseQuery(Query, {
		variables: {
			rootId,
		},
	});

	const root = data.root;
	if (root.kind !== "SERIES") {
		return <MediaHeader media={root} />;
	}

	const sortedSeasons = [...root.seasons].sort((a, b) => {
		const seasonA = readFragment(SeasonCardFrag, a);
		const seasonB = readFragment(SeasonCardFrag, b);
		return seasonA.order - seasonB.order || seasonA.seasonNumber - seasonB.seasonNumber;
	});

	return (
		<div className="pt-6">
			<MediaHeader media={root} />
			<div className="container mx-auto py-6">
				<h2 className="font-semibold text-zinc-200 mb-2">Seasons</h2>
				{sortedSeasons.length > 0 ? (
					<div className="flex flex-wrap gap-4">
						{sortedSeasons.map((season) => (
							<SeasonCard
								key={season.id}
								season={season}
								path={`/library/${root.libraryId}/root/${root.id}/season/${season.id}`}
							/>
						))}
					</div>
				) : (
					<div className="text-zinc-400">No seasons found for this series.</div>
				)}
			</div>
		</div>
	);
}
