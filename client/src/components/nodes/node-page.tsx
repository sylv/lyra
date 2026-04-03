import { useEffect, useState, type FC } from "react";
import { useQuery } from "urql";
import { graphql } from "../../@generated/gql";
import type { NodeFilter } from "../../@generated/gql/graphql";
import { EpisodeCard } from "../episode-card";
import { DisplayKind, type PageVariables } from "./node-list";
import { NodePosterDetail } from "./node-poster-detail";
import { ViewLoader } from "../view-loader";
import { Spinner } from "../ui/spinner";

interface NodePageProps {
	displayKind: DisplayKind;
	filter: NodeFilter;
	perPage: number;
	variables: PageVariables;
	isLast: boolean;
	isFirst: boolean;
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

export const NodePage: FC<NodePageProps> = ({
	displayKind,
	perPage,
	filter,
	variables,
	isFirst,
	isLast,
	onLoadMore,
}) => {
	const [firstLoad, setFirstLoad] = useState(isFirst);
	const [{ data }] = useQuery({
		query: Query,
		context: { suspense: firstLoad },
		variables: { after: variables.after, first: perPage, filter },
	});

	useEffect(() => {
		if (data) setFirstLoad(false);
	}, [data]);

	return (
		<>
			{!data && <Spinner />}
			{data?.nodeList.edges.map((edge) =>
				displayKind === DisplayKind.Episode ? (
					<EpisodeCard episode={edge.node} key={edge.node.id} />
				) : (
					<NodePosterDetail node={edge.node} key={edge.node.id} />
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
