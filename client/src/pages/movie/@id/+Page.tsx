import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader, MediaHeaderFrag } from "../../../components/media-header";
import { graphql } from "gql.tada";
import { useSuspenseQuery } from "@apollo/client";

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
	const { data } = useSuspenseQuery(Query, {
		variables: {
			mediaId: mediaId,
		},
	});

	return <MediaHeader media={data.media} />;
}
