import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { LibrariesQuery, LibraryManager } from "../components/library-manager";

export const Route = createFileRoute("/settings/libraries")({
	component: RouteComponent,
});

function RouteComponent() {
	const { data: librariesData, loading } = useQuery(LibrariesQuery);
	const libraries = librariesData?.libraries ?? [];

	return (
		<section className="space-y-4">
			<div>
				<h3>Libraries</h3>
				<p className="text-sm text-zinc-400">
					Add scan roots after setup, update existing libraries, and review when each one was last scanned.
				</p>
			</div>
			<LibraryManager libraries={libraries} loading={loading} />
		</section>
	);
}
