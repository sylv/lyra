import { createFileRoute } from "@tanstack/react-router";
import { NodeList } from "../components/nodes/node-list";
import { useTitle } from "../hooks/use-title";

export const Route = createFileRoute("/")({
	component: HomeRoute,
});

function HomeRoute() {
	useTitle("Home");
	return <NodeList type="movies_posters" />;
}
