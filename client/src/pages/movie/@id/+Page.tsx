import { useQuery } from "@apollo/client";
import { graphql } from "gql.tada";
import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader, MediaHeaderFrag, MediaHeaderSkeleton } from "../../../components/media-header";

const Query = graphql(
	`
	query GetMediaById($mediaId: Int!) {
		media(mediaId: $mediaId) {
			...MediaHeader
		}
	}
`,
	[MediaHeaderFrag],
);

export default function Page() {
	const pageContext = usePageContext();
	const mediaId = +pageContext.routeParams.id;
	const { loading, data } = useQuery(Query, {
		variables: {
			mediaId: mediaId,
		},
	});

	if (loading || !data) {
		return <MediaHeaderSkeleton />;
	}

	return <MediaHeader media={data.media} />;
}
