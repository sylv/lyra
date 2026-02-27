import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { DirectoryPicker } from "@/components/directory-picker";

export const Route = createFileRoute("/playground")({
	component: PlaygroundRoute,
});

function PlaygroundRoute() {
	const [, setPath] = useState<string | null>("/");

	return (
		<div className="p-6">
			<DirectoryPicker onPathChange={setPath} />
		</div>
	);
}
