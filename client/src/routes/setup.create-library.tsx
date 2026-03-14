import { createFileRoute } from "@tanstack/react-router";
import { SetupPage } from "../components/setup/setup-page";
import { CreateLibraryStep } from "../components/setup/steps/create-library-step";
import { useSetup } from "../components/setup/setup-wrapper";

export const Route = createFileRoute("/setup/create-library")({
	component: SetupCreateLibraryRoute,
});

function SetupCreateLibraryRoute() {
	const { state } = useSetup();

	if (state?.state !== "create_first_library") {
		return null;
	}

	return (
		<SetupPage title="Let's get you sorted" description="Set up your media libraries">
			<CreateLibraryStep />
		</SetupPage>
	);
}
