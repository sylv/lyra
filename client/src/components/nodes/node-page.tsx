import { useQuery } from "@apollo/client/react";
import { type FC } from "react";
import { graphql } from "../../@generated/gql";
import type { NodeFilter } from "../../@generated/gql/graphql";
import { EpisodeCard } from "../episode-card";
import { DisplayKind, type PageVariables } from "./node-list";
import { NodePoster } from "./node-poster";
import { ViewLoader } from "../view-loader";
import { Spinner } from "../ui/spinner";

interface NodePageProps {
	displayKind: DisplayKind;
	filter: NodeFilter;
	perPage: number;
	variables: PageVariables;
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

export const NodePage: FC<NodePageProps> = ({ displayKind, perPage, filter, variables, isLast, onLoadMore }) => {
	const { data } = useQuery(Query, {
		variables: { after: variables.after, first: perPage, filter },
	});

	return (
		<>
			{!data && <Spinner />}
			{data?.nodeList.edges.map((edge) =>
				displayKind === DisplayKind.Episode ? (
					<EpisodeCard episode={edge.node} key={edge.node.id} />
				) : (
					<NodePoster node={edge.node} key={edge.node.id} />
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
