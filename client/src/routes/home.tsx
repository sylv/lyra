import { NodeList } from "../components/nodes/node-list";
import { useTitle } from "../hooks/use-title";

export function HomeRoute() {
	useTitle("Home");
	return <NodeList type="movies_posters" />;
}
