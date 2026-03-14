import { createFileRoute } from "@tanstack/react-router";
import { SetupPage } from "../components/setup/setup-page";
import { CreateAccountForm } from "../components/setup/steps/create-account-form";
import { useSetup } from "../components/setup/setup-wrapper";

export const Route = createFileRoute("/setup/create-account")({
	component: SetupCreateAccountRoute,
});

function SetupCreateAccountRoute() {
	const { state } = useSetup();

	if (state?.state !== "create_first_user") {
		return null;
	}

	return (
		<SetupPage title="Let's get you sorted" description="Create your first account">
			<CreateAccountForm />
		</SetupPage>
	);
}
