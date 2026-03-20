import { useQuery } from "@apollo/client/react";
import { LibrariesQuery, LibraryManager } from "../libraries";
import { SetupStep } from "./setup-step";
import { useSetup } from "./setup-wrapper";

export function CreateLibraryStep() {
	const { refresh } = useSetup();
	const { data: librariesData, loading } = useQuery(LibrariesQuery);
	const libraries = librariesData?.libraries || [];

	return (
		<SetupStep loading={loading} disabled={libraries.length === 0} onSubmit={() => refresh()} centered={false}>
			<LibraryManager libraries={libraries} loading={loading} className="mb-6" />
		</SetupStep>
	);
}
