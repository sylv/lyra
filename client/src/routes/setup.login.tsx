import { createFileRoute } from "@tanstack/react-router";
import { SetupPage } from "../components/setup/setup-page";
import { LoginForm } from "../components/setup/steps/login-form";
import { useSetup } from "../components/setup/setup-wrapper";

export const Route = createFileRoute("/setup/login")({
	component: SetupLoginRoute,
});

function SetupLoginRoute() {
	const { state } = useSetup();

	if (state?.state !== "login") {
		return null;
	}

	return (
		<SetupPage title="Let's get you sorted" description="Login to your account">
			<LoginForm />
		</SetupPage>
	);
}
