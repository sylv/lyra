import { useCallback, useEffect, useRef, useState, type FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import { MediaPoster } from "./media-poster";
import { Spinner } from "./ui/spinner";
import { ViewLoader } from "./view-loader";

const Fragment = graphql(`
	fragment MediaList on Node {
		id
		...MediaPoster
	}
`);

interface MediaListProps {
	media?: FragmentType<typeof Fragment>[];
	loading: boolean;
	onLoadMore?: () => void;
}

const POSTER_WIDTH = 185;
const GAP_SIZE = 16;

export const MediaList: FC<MediaListProps> = ({ media: mediaRaw, loading, onLoadMore }) => {
	const media = mediaRaw ? unmask(Fragment, mediaRaw) : [];
	const containerRef = useRef<HTMLDivElement>(null);
	const [columns, setColumns] = useState<number | null>(null);

	const calculateLayout = useCallback(() => {
		const containerWidth = containerRef.current?.clientWidth || 0;
		if (containerWidth === 0) return 1;
		setColumns(Math.max(1, Math.ceil(containerWidth / (POSTER_WIDTH + GAP_SIZE))));
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
		<div ref={containerRef} className="w-full relative mb-24">
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
