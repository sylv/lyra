import { useQuery } from "@apollo/client/react";
import { graphql } from "gql.tada";
import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../components/media-header";

const Query = graphql(
	`
	query GetMediaById($rootId: String!) {
		root(rootId: $rootId) {
			...MediaHeader
		}
	}
`,
	[MediaHeaderFrag],
);

export default function Page() {
	const pageContext = usePageContext();
	const rootId = pageContext.routeParams.id;
	const { loading, data } = useQuery(Query, {
		variables: {
			rootId,
		},
	});

	if (loading || !data) {
		return <MediaHeaderSkeleton />;
	}

	return <MediaHeader media={data.root} />;
}
