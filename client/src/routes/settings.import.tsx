import { useState } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { Button } from "../components/button";
import { PlexImportModal } from "../components/import/plex-import-modal";

export const Route = createFileRoute("/settings/import")({
	component: RouteComponent,
});

function RouteComponent() {
	const [isPlexImportOpen, setIsPlexImportOpen] = useState(false);

	return (
		<>
			<div className="flex flex-col gap-3 md:flex-row md:items-center">
				<div className="flex-1">
					<h3>Plex</h3>
					<p className="text-zinc-400 text-sm">Import watch progress from Plex</p>
				</div>
				<Button className="bg-[#e5a00d] text-black hover:bg-[#e5a00d]" onClick={() => setIsPlexImportOpen(true)}>
					Import from Plex
				</Button>
			</div>
			<PlexImportModal open={isPlexImportOpen} onOpenChange={setIsPlexImportOpen} />
		</>
	);
}
