import { useState } from "react";
import { Button, ButtonStyle } from "../components/button";
import { PlexImportModal } from "../components/import/plex-import-modal";

export function SettingsImportRoute() {
	const [isPlexImportOpen, setIsPlexImportOpen] = useState(false);

	return (
		<>
			<div className="flex flex-col gap-3 md:flex-row md:items-center">
				<div className="flex-1">
					<h3>Plex</h3>
					<p className="text-sm text-zinc-400">Import watch progress from Plex</p>
				</div>
				<Button style={ButtonStyle.Plex} onClick={() => setIsPlexImportOpen(true)}>
					Import from Plex
				</Button>
			</div>
			<PlexImportModal open={isPlexImportOpen} onOpenChange={setIsPlexImportOpen} />
		</>
	);
}
