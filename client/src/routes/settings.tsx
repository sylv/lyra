import { useState } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { PlexImportModal } from "../components/import/plex-import-modal";
import { Button } from "../components/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../components/ui/tabs";
import { useTitle } from "../hooks/use-title";

export const Route = createFileRoute("/settings")({
	component: RouteComponent,
});

function RouteComponent() {
	const buildDate = new Date(__BUILD_TIME__).toLocaleString();
	const [isPlexImportOpen, setIsPlexImportOpen] = useState(false);

	useTitle("Settings");

	return (
		<div className="pt-6">
			<Tabs defaultValue="about" className="w-full">
				<TabsList>
					<TabsTrigger value="about">About</TabsTrigger>
					<TabsTrigger value="import">Import</TabsTrigger>
				</TabsList>
				<div className="rounded bg-zinc-400/10 p-3 min-h-[70vh]">
					<TabsContent value="about">
						<p className="text-sm text-zinc-400">
							Based on {__BRANCH__} {__REVISION__}, built on {buildDate}.
						</p>
					</TabsContent>
					<TabsContent value="import">
						<div className="flex flex-col gap-3 p-4 md:flex-row md:items-center">
							<div className="flex-1">
								<h3 className="text-sm font-semibold">Plex</h3>
								<p className="text-xs text-zinc-400">Import watch progress from Plex</p>
							</div>
							<Button className="bg-[#e5a00d] text-black hover:bg-[#e5a00d]" onClick={() => setIsPlexImportOpen(true)}>
								Import from Plex
							</Button>
						</div>
					</TabsContent>
				</div>
			</Tabs>
			<PlexImportModal open={isPlexImportOpen} onOpenChange={setIsPlexImportOpen} />
		</div>
	);
}
