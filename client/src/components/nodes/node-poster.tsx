import { Link } from "@tanstack/react-router";
import type React from "react";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../../@generated/gql";
import type { NodePosterFragment } from "../../@generated/gql/graphql";
import { formatReleaseYear } from "../../lib/format-release-year";
import { getPathForNode } from "../../lib/getPathForMedia";
import { cn } from "../../lib/utils";
import { Image, ImageType } from "../image";
import { PlayWrapper } from "../play-wrapper";
import { UnplayedItemsTab } from "../unplayed-items-tab";

interface NodePosterProps {
	node: FragmentType<typeof Fragment>;
	className?: string;
	style?: React.CSSProperties;
}

const Fragment = graphql(`
	fragment NodePoster on Node {
		id
		name
		kind
		libraryId
		properties {
			posterImage {
				...ImageAsset
			}
			releasedAt
			endedAt
		}
		nextPlayable {
			id
			watchProgress {
				progressPercent
				completed
				updatedAt
			}
		}
		unplayedCount
		seasonCount
		episodeCount
		...GetPathForNode
	}
`);

export const NodePoster: FC<NodePosterProps> = ({ node: nodeRaw, className, style }) => {
	const node = unmask(Fragment, nodeRaw);
	const path = getPathForNode(node);
	const detail = getPosterDetail(node);

	return (
		<div className={cn("flex flex-col gap-2 overflow-hidden select-none", className)} style={style}>
			<PlayWrapper
				itemId={node.nextPlayable?.id ?? node.id}
				path={path}
				watchProgress={node.nextPlayable?.watchProgress ?? null}
			>
				<Image type={ImageType.Poster} asset={node.properties.posterImage} alt={node.name} className="w-full" />
				<UnplayedItemsTab>{node.unplayedCount}</UnplayedItemsTab>
			</PlayWrapper>
			<Link to={path} className="block w-full truncate text-sm group">
				<span className="group-hover:underline">{node.name}</span>
				{detail && <p className="text-xs text-zinc-500 -mt-0.5">{detail}</p>}
			</Link>
		</div>
	);
};

const getPosterDetail = (node: NodePosterFragment): string | number | null => {
	if (node.kind === "SERIES") {
		if (node.seasonCount > 0) return `${node.seasonCount} ${node.seasonCount === 1 ? "season" : "seasons"}`;
		if (node.episodeCount > 0) return `${node.episodeCount} ${node.episodeCount === 1 ? "episode" : "episodes"}`;
	}

	if (!node.properties.releasedAt) return null;
	return formatReleaseYear(node.properties.releasedAt, node.properties.endedAt ?? null) ?? null;
};
