import { gql } from "@apollo/client";
import { useMutation } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { Button, ButtonStyle } from "../components/button";
import { Input } from "../components/input";
import { InputOtp } from "../components/input-otp";
import { SetupPage } from "../components/setup/setup-page";
import { SetupStep } from "../components/setup/setup-step";
import { useSetup } from "../components/setup/setup-wrapper";
import { useTitle } from "../hooks/use-title";

const SIGNUP_MUTATION = gql`
	mutation Signup($username: String!, $password: String!) {
		signup(username: $username, password: $password) {
			id
			username
		}
	}
`;

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

function CreateAccountForm() {
	const { refresh } = useSetup();
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	const [waitingForCode, setWaitingForCode] = useState(true);
	const [setupCode, setSetupCode] = useState<number | null>(null);

	const [signup] = useMutation(SIGNUP_MUTATION, {});

	useTitle("Create your account");

	const handleSubmit = async () => {
		setError(null);

		if (!setupCode) {
			setError("Invalid or incomplete code");
			return;
		}

		if (waitingForCode) {
			setWaitingForCode(false);
			return;
		}

		if (!username.trim()) {
			setError("Username is required");
			return;
		}

		if (!password.trim()) {
			setError("Password is required");
			return;
		}

		if (password !== confirmPassword) {
			setError("Passwords do not match");
			return;
		}

		try {
			setLoading(true);
			await signup({
				variables: {
					username: username.trim(),
					password,
				},
				context: {
					headers: {
						"x-setup-code": setupCode.toString(),
					},
				},
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
		<SetupStep
			loading={loading}
			disabled={loading}
			onSubmit={handleSubmit}
			error={error}
			footer={
				waitingForCode ? undefined : (
					<Button style={ButtonStyle.Transparent} onClick={() => setWaitingForCode(true)}>
						Back
					</Button>
				)
			}
		>
			{waitingForCode ? (
				<fieldset>
					<InputOtp onChange={setSetupCode} />
					<p className="text-zinc-600 text-xs mt-3 text-center">Enter the code from Lyra's startup logs.</p>
				</fieldset>
			) : (
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
						<Input
							type="password"
							placeholder="Confirm Password"
							value={confirmPassword}
							onChange={(e) => setConfirmPassword(e.target.value)}
							required
						/>
					</fieldset>
				</form>
			)}
		</SetupStep>
	);
}
