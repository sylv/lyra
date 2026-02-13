import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../components/media-header";

const Query = graphql(
	`
	query GetMediaById($nodeId: String!) {
		node(nodeId: $nodeId) {
			...MediaHeader
		}
	}
`,
	[MediaHeaderFrag],
);

export default function Page() {
	const pageContext = usePageContext();
	const nodeId = pageContext.routeParams.id;
	const { loading, data } = useQuery(Query, {
		variables: {
			nodeId: nodeId,
		},
	});

	if (loading || !data) {
		return <MediaHeaderSkeleton />;
	}

	return <MediaHeader media={data.node} />;
}
