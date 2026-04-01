import { useQuery } from "urql";
import { LibrariesQuery, LibraryManager } from "../components/settings/libraries";

export function SettingsLibrariesRoute() {
	const [{ data: librariesData, fetching }] = useQuery({ query: LibrariesQuery });
	const libraries = librariesData?.libraries ?? [];

	return (
		<section className="space-y-4">
			<div>
				<h3>Libraries</h3>
				<p className="mt-1 text-sm text-zinc-400">Create and manage libraries to organize your media</p>
			</div>
			<LibraryManager libraries={libraries} loading={fetching} />
		</section>
	);
}
