import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { DirectoryPicker } from "@/components/directory-picker";
import { useTitle } from "../hooks/use-title";

export const Route = createFileRoute("/playground")({
	component: PlaygroundRoute,
});

function PlaygroundRoute() {
	const [, setPath] = useState<string | null>("/");

	useTitle("Playground");

	return (
		<div className="p-6">
			<DirectoryPicker onPathChange={setPath} />
		</div>
	);
}
