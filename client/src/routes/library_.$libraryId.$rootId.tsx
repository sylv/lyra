import { useSuspenseQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { graphql, readFragment } from "gql.tada";
import { Image, ImageAssetFrag, ImageType } from "@/components/image";
import { PlayWrapper } from "@/components/play-wrapper";
import { SeasonCard, SeasonCardFrag } from "@/components/season-card";
import { UnplayedItemsTab } from "@/components/unplayed-items-tab";
import { useDynamicBackground } from "@/hooks/use-background";
import { formatReleaseYear } from "@/lib/format-release-year";

const Query = graphql(
	`
	query GetRootById($rootId: String!) {
		root(rootId: $rootId) {
			id
			kind
			name
			libraryId
			seasons {
				id
				...SeasonCard
			}
			properties {
				posterImage {
					...ImageAsset
				}
				backgroundImage {
					...ImageAsset
				}
				releasedAt
				endedAt
				runtimeMinutes
				description
			}
			playableItem {
				id
			}
			watchProgress {
				progressPercent
				updatedAt
			}
			unplayedItems
		}
	}
`,
	[SeasonCardFrag, ImageAssetFrag],
);

export const Route = createFileRoute("/library_/$libraryId/$rootId")({
	component: RootRoute,
});

function RootRoute() {
	const { rootId } = Route.useParams();
	const { data } = useSuspenseQuery(Query, {
		variables: {
			rootId,
		},
	});

	const root = data.root;
	const rootPath = `/library/${root.libraryId}/${root.id}`;

	const dynamicAsset = root.properties.backgroundImage || root.properties.posterImage;
	useDynamicBackground(dynamicAsset);

	const header = (
		<div className="flex gap-6 container mx-auto">
			<PlayWrapper itemId={root.playableItem?.id} path={rootPath} watchProgress={root.watchProgress}>
				<Image type={ImageType.Poster} asset={root.properties.posterImage} alt={root.name} className="h-96" />
				<UnplayedItemsTab count={root.unplayedItems} />
			</PlayWrapper>
			<div className="flex flex-col gap-2 justify-between">
				<div className="flex flex-col gap-2 mt-3">
					<span className="text-sm text-zinc-400 -mb-2">
						{formatReleaseYear(root.properties.releasedAt, root.properties.endedAt ?? null)}
					</span>
					<h1 className="text-2xl font-bold">{root.name}</h1>
					{root.properties.runtimeMinutes && (
						<p className="text-sm text-zinc-400">{root.properties.runtimeMinutes} minutes</p>
					)}
					<p className="text-sm text-zinc-400">{root.properties.description || "No description for this"}</p>
				</div>
			</div>
		</div>
	);

	if (root.kind !== "SERIES") {
		return header;
	}

	const sortedSeasons = [...root.seasons].sort((a, b) => {
		const seasonA = readFragment(SeasonCardFrag, a);
		const seasonB = readFragment(SeasonCardFrag, b);
		return seasonA.order - seasonB.order || seasonA.seasonNumber - seasonB.seasonNumber;
	});

	return (
		<div className="pt-6">
			{header}
			<div className="container mx-auto py-6">
				<h2 className="font-semibold text-zinc-200 mb-2">Seasons</h2>
				{sortedSeasons.length > 0 ? (
					<div className="flex flex-wrap gap-4">
						{sortedSeasons.map((season) => (
							<SeasonCard key={season.id} season={season} path={`/library/${root.libraryId}/${root.id}/${season.id}`} />
						))}
					</div>
				) : (
					<div className="text-zinc-400">No seasons found for this series.</div>
				)}
			</div>
		</div>
	);
}
