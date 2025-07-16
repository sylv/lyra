import { usePageContext } from "vike-react/usePageContext";
import { MediaHeader } from "../../../components/media-header";
import { trpc } from "../../trpc";

export default function Page() {
	const pageContext = usePageContext();
	const mediaId = +pageContext.routeParams.id;
	const [details] = trpc.get_media_by_id.useSuspenseQuery({
		media_id: mediaId,
	});

	return (
		<>
			<MediaHeader details={details} />
		</>
	);
}
