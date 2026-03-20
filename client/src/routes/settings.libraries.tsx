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
				<p className="mt-1 text-sm text-zinc-400">Create and manage libraries to organize your media</p>
			</div>
			<LibraryManager libraries={libraries} loading={loading} />
		</section>
	);
}
