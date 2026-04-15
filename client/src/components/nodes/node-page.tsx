import { useMemo, type FC } from "react";
import { useQuery } from "urql";
import { graphql } from "../../@generated/gql";
import type { NodePageQueryVariables } from "../../@generated/gql/graphql";
import { Spinner } from "../ui/spinner";
import { ViewLoader } from "../view-loader";
import { DisplayKind } from "./node-list";
import { EpisodePosterDetail } from "./node-list-episode-detail";
import { NodePosterDetail } from "./node-poster-detail";

interface NodePageProps {
	displayKind: DisplayKind;
	variables: NodePageQueryVariables;
	isFirst: boolean;
	isLast: boolean;
	onLoadMore: (after: string) => void;
}

const Query = graphql(`
	query NodePage($after: String, $first: Int!, $filter: NodeFilter!) {
		nodeList(after: $after, first: $first, filter: $filter) {
			edges {
				node {
					id
					...NodePoster
					...EpisodeCard
				}
			}
			pageInfo {
				endCursor
				hasNextPage
			}
		}
	}
`);

export const NodePage: FC<NodePageProps> = ({ displayKind, variables, isFirst, isLast, onLoadMore }) => {
	const queryContext = useMemo(() => ({ suspense: isFirst }), [isFirst]);
	const [{ data }] = useQuery({
		query: Query,
		context: queryContext,
		variables: variables,
	});

	return (
		<>
			{!data && <Spinner />}
			{data?.nodeList.edges.map((edge) =>
				displayKind === DisplayKind.Episode ? (
					<EpisodePosterDetail key={edge.node.id} episode={edge.node} />
				) : (
					<NodePosterDetail key={edge.node.id} node={edge.node} />
				),
			)}
			{isLast && data?.nodeList.pageInfo.hasNextPage && data.nodeList.pageInfo.endCursor && (
				<ViewLoader
					onLoadMore={() => {
						if (!data.nodeList.pageInfo.endCursor) return;
						onLoadMore(data.nodeList.pageInfo.endCursor);
					}}
				/>
			)}
		</>
	);
};
