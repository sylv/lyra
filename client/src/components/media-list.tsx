import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { useCallback, useEffect, useRef, useState, type FC } from "react";
import { MediaPoster, MediaPosterFrag } from "./media-poster";
import { Spinner } from "./ui/spinner";
import { ViewLoader } from "./view-loader";

export const MediaListFrag = graphql(
	`
    fragment MediaList on Node {
        id
        ...MediaPoster
    }
`,
	[MediaPosterFrag],
);

interface MediaListProps {
	media?: FragmentOf<typeof MediaListFrag>[];
	loading: boolean;
	onLoadMore?: () => void;
}

const POSTER_WIDTH = 185;
const GAP_SIZE = 16;

export const MediaList: FC<MediaListProps> = ({ media: mediaRaw, loading, onLoadMore }) => {
	const media = mediaRaw ? readFragment(MediaListFrag, mediaRaw) : [];
	const containerRef = useRef<HTMLDivElement>(null);
	const [columns, setColumns] = useState<number | null>(null);

	/**
	 * Calculates the optimal number of posters per row and their exact width.
	 * It iterates downwards from a maximum possible number of posters, finding the
	 * first count that allows posters to be sized within the defined WIDTH_THRESHOLD.
	 * This maximizes the number of posters in a row while respecting size constraints.
	 */
	const calculateLayout = useCallback(() => {
		const containerWidth = containerRef.current?.clientWidth || 0;
		if (containerWidth === 0) return 1; // Default to 1 column if width is not available
		const cols = Math.max(1, Math.ceil(containerWidth / (POSTER_WIDTH + GAP_SIZE)));
		setColumns(cols);
	}, []);

	useEffect(() => {
		calculateLayout();
		window.addEventListener("resize", calculateLayout);
		return () => window.removeEventListener("resize", calculateLayout);
	}, [calculateLayout]);

	if (!media || (media.length === 0 && loading)) {
		return (
			<div ref={containerRef} className="mr-6 w-full h-dvh flex items-center justify-center">
				<Spinner className="size-6" />
			</div>
		);
	}

	return (
		<div ref={containerRef} className="w-full relative mr-6 mb-24">
			<div
				className="grid"
				style={{ gridTemplateColumns: `repeat(${columns}, 1fr)`, columnGap: GAP_SIZE, rowGap: GAP_SIZE }}
			>
				{media.map((mediaItem) => (
					<MediaPoster media={mediaItem} key={mediaItem.id} />
				))}
			</div>
			<ViewLoader onLoadMore={onLoadMore} />
		</div>
	);
};
