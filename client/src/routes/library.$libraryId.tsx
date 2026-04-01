import { NodeList } from "@/components/nodes/node-list";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/library/$libraryId")({
	component: LibraryRoute,
});

function LibraryRoute() {
	const { libraryId } = Route.useParams();
	return <NodeList type="movies_posters" filterOverride={{ libraryId }} />;
}
