import { useQuery } from "@apollo/client/react";
import { graphql, readFragment } from "gql.tada";
import { Fragment } from "react/jsx-runtime";
import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../../../components/media-header";
import { SeasonCard, SeasonCardFrag, SeasonCardSkeleton } from "../../../../../components/season-card";

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
	const { data, loading } = useQuery(Query, {
		variables: {
			rootId,
		},
	});

	if (loading || !data) {
		return (
			<Fragment>
				<MediaHeaderSkeleton />
				<div className="container mx-auto py-6">
					<div className="mb-4">
						<div className="h-6 w-24 rounded-md bg-zinc-700/50" />
					</div>
					<div className="grid gap-4 grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-7">
						{Array.from({ length: 7 }).map((_, index) => (
							<SeasonCardSkeleton key={`season-skeleton-${index}`} />
						))}
					</div>
				</div>
			</Fragment>
		);
	}

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
		<Fragment>
			<MediaHeader media={root} />
			<div className="container mx-auto py-6">
				<h2 className="text-xl font-semibold text-zinc-200 mb-4">Seasons</h2>
				{sortedSeasons.length > 0 ? (
					<div className="grid gap-4 grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-7">
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
		</Fragment>
	);
}
