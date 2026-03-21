import { type FC } from "react";
import { graphql, unmask, type FragmentType } from "../../@generated/gql";
import { NodePoster } from "./node-poster";
import { Spinner } from "../ui/spinner";
import { ViewLoader } from "../view-loader";

const Fragment = graphql(`
	fragment NodeList on Node {
		id
		...NodePoster
	}
`);

interface NodeListProps {
	nodes?: FragmentType<typeof Fragment>[];
	loading: boolean;
	onLoadMore?: () => void;
}

const POSTER_WIDTH = 185;
const GAP_SIZE = 16;

export const NodeList: FC<NodeListProps> = ({ nodes: nodesRaw, loading, onLoadMore }) => {
	const nodes = nodesRaw ? unmask(Fragment, nodesRaw) : [];

	if (!nodes || (nodes.length === 0 && loading)) {
		return (
			<div className="mr-6 w-full h-dvh flex items-center justify-center">
				<Spinner className="size-6" />
			</div>
		);
	}

	return (
		<div className="w-full relative mb-24">
			<div
				className="grid"
				style={{
					gridTemplateColumns: `repeat(auto-fill, minmax(${POSTER_WIDTH}px, 1fr))`,
					columnGap: GAP_SIZE,
					rowGap: GAP_SIZE,
				}}
			>
				{nodes.map((node) => (
					<NodePoster node={node} key={node.id} />
				))}
			</div>
			<ViewLoader onLoadMore={onLoadMore} />
		</div>
	);
};
