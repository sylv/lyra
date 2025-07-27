import { graphql, readFragment, type FragmentOf } from "gql.tada";
import { useEffect, useMemo, useRef, useState, type FC } from "react";
import { MediaPoster, MediaPosterFrag } from "./media-poster";

export const MediaListFrag = graphql(
	`
    fragment MediaList on Media {
        id
        ...MediaPoster
    }
`,
	[MediaPosterFrag],
);

interface MediaListProps {
	media: FragmentOf<typeof MediaListFrag>[];
}

const POSTER_WIDTH = 185;
const GAP_SIZE = 28;

export const MediaList: FC<MediaListProps> = ({ media: mediaRaw }) => {
	const media = readFragment(MediaListFrag, mediaRaw);
	const containerRef = useRef<HTMLDivElement>(null);
	const [posterWidth, setPosterWidth] = useState(0);
	const [postersPerRow, setPostersPerRow] = useState(0);

	/**
	 * Calculates the optimal number of posters per row and their exact width.
	 * It iterates downwards from a maximum possible number of posters, finding the
	 * first count that allows posters to be sized within the defined WIDTH_THRESHOLD.
	 * This maximizes the number of posters in a row while respecting size constraints.
	 */
	const calculateLayout = () => {
		if (!containerRef.current) return;

		const containerWidth = containerRef.current.getBoundingClientRect().width;
		if (containerWidth === 0) return;

		if (containerWidth <= POSTER_WIDTH) {
			setPosterWidth(containerWidth);
			setPostersPerRow(1);
			return;
		}

		let perRow = 1;
		while (true) {
			const requiredWidth = perRow * (POSTER_WIDTH + GAP_SIZE);
			if (requiredWidth > containerWidth) {
				perRow--;
				setPosterWidth(containerWidth / perRow - GAP_SIZE);
				setPostersPerRow(perRow);
				break;
			}

			perRow++;
		}
	};

	useEffect(() => {
		// Run calculation on initial mount
		calculateLayout();

		// Set up a ResizeObserver to recalculate whenever the container's size changes.
		const resizeObserver = new ResizeObserver(() => {
			calculateLayout();
		});

		if (containerRef.current) {
			resizeObserver.observe(containerRef.current);
		}

		// Cleanup observer on component unmount.
		return () => {
			resizeObserver.disconnect();
		};
	}, []); // Empty dependency array ensures this runs only once on mount.

	// Memoize the row calculation to avoid re-computing on every render.
	const rows = useMemo(() => {
		if (postersPerRow === 0 || !media) return [];
		const newRows: (typeof media)[] = [];
		for (let i = 0; i < media.length; i += postersPerRow) {
			newRows.push(media.slice(i, i + postersPerRow));
		}
		return newRows;
	}, [media, postersPerRow]);

	// This initial render with opacity-0 is a trick to measure the container's width
	// without causing a visible layout shift before the real content is rendered.
	if (posterWidth === 0 || postersPerRow === 0) {
		return <div ref={containerRef} className="w-full h-1 opacity-0" />;
	}

	return (
		<div ref={containerRef} className="w-full transition-opacity duration-300 ease-in-out opacity-100">
			{rows.map((row, index) => (
				<div key={`row-${index}-${row.length}`} className="flex mb-6" style={{ gap: GAP_SIZE }}>
					{row.map((mediaItem) => (
						<MediaPoster media={mediaItem} key={mediaItem.id} style={{ width: posterWidth }} />
					))}
				</div>
			))}
		</div>
	);
};
