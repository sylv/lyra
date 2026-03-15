import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { Input } from "../components/input";
import { SetupPage } from "../components/setup/setup-page";
import { SetupStep } from "../components/setup/setup-step";
import { useSetup } from "../components/setup/setup-wrapper";
import { useTitle } from "../hooks/use-title";

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

function LoginForm() {
	const { refresh } = useSetup();
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	useTitle("Sign in");

	const handleSubmit = async () => {
		setError(null);

		if (!username.trim()) {
			setError("Username is required");
			return;
		}

		if (!password.trim()) {
			setError("Password is required");
			return;
		}

		try {
			setLoading(true);
			await fetch("/api/login", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					username: username.trim(),
					password,
				}),
			});

			await refresh();
			setLoading(false);
		} catch (error: any) {
			// todo: handle 401s
			setError(error.message);
		} finally {
			setLoading(false);
		}
	};

	return (
		<SetupStep loading={loading} disabled={loading} onSubmit={handleSubmit} error={error}>
			<form id="create-account-form" onSubmit={handleSubmit}>
				<fieldset className="flex flex-col gap-2">
					<Input
						type="text"
						placeholder="Username"
						value={username}
						onChange={(e) => setUsername(e.target.value)}
						required
					/>
					<Input
						type="password"
						placeholder="Password"
						value={password}
						onChange={(e) => setPassword(e.target.value)}
						required
					/>
				</fieldset>
			</form>
		</SetupStep>
	);
}
