import { NodeList } from "@/components/nodes/node-list";
import { OrderBy } from "../@generated/gql/graphql";
import { useParams } from "react-router";

export function LibraryRoute() {
	const { libraryId } = useParams<{ libraryId: string }>();
	if (!libraryId) {
		return null;
	}
	return <NodeList type="movies_series" defaultOrderBy={OrderBy.ReleasedAt} filterOverride={{ libraryId }} />;
}
