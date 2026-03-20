import { useMutation } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { graphql } from "../@generated/gql";
import { Button, ButtonStyle } from "../components/button";
import { Input } from "../components/input";
import { InputOtp } from "../components/input-otp";
import { SetupPage } from "../components/settings/setup/setup-page";
import { SetupStep } from "../components/settings/setup/setup-step";
import { useSetup } from "../components/settings/setup/setup-wrapper";
import { useTitle } from "../hooks/use-title";

const SIGNUP_MUTATION = graphql(`
	mutation Signup($username: String!, $password: String!, $inviteCode: String) {
		signup(username: $username, password: $password, inviteCode: $inviteCode) {
			id
			username
		}
	}
`);

export const Route = createFileRoute("/setup/create-account")({
	component: SetupCreateAccountRoute,
});

function SetupCreateAccountRoute() {
	const { state } = useSetup();

	if (state?.state !== "create_first_user" && state?.state !== "create_invited_user") {
		return null;
	}

	return (
		<SetupPage
			title="Let's get you sorted"
			description={
				state.state === "create_invited_user" ? "Finish setting up your invited account" : "Create your first account"
			}
		>
			<CreateAccountForm
				key={state.state === "create_invited_user" ? state.invite_code : "create-first-user"}
				mode={state.state}
				initialUsername={state.state === "create_invited_user" ? state.username : ""}
				inviteCode={state.state === "create_invited_user" ? state.invite_code : null}
			/>
		</SetupPage>
	);
}

function CreateAccountForm({
	mode,
	initialUsername,
	inviteCode,
}: {
	mode: "create_first_user" | "create_invited_user";
	initialUsername: string;
	inviteCode: string | null;
}) {
	const { refresh } = useSetup();
	const isInviteFlow = mode === "create_invited_user";
	const [username, setUsername] = useState(initialUsername);
	const [usernameLocked, setUsernameLocked] = useState(isInviteFlow);
	const [password, setPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	const [waitingForCode, setWaitingForCode] = useState(!isInviteFlow);
	const [setupCode, setSetupCode] = useState<number | null>(null);

	const [signup] = useMutation(SIGNUP_MUTATION, {});

	useEffect(() => {
		setUsername(initialUsername);
		setUsernameLocked(isInviteFlow);
		setWaitingForCode(!isInviteFlow);
		setSetupCode(null);
	}, [initialUsername, isInviteFlow]);

	useTitle(isInviteFlow ? "Finish account setup" : "Create your account");

	const handleSubmit = async () => {
		setError(null);

		if (!isInviteFlow && !setupCode) {
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
					inviteCode: inviteCode,
				},
				context: isInviteFlow
					? undefined
					: {
							headers: {
								"x-setup-code": setupCode!.toString(),
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
				<form
					id="create-account-form"
					onSubmit={(event) => {
						event.preventDefault();
						void handleSubmit();
					}}
				>
					<fieldset className="flex flex-col gap-2">
						<div className="flex items-center gap-2">
							<Input
								type="text"
								placeholder="Username"
								value={username}
								onChange={(e) => setUsername(e.target.value)}
								disabled={usernameLocked}
								required
								className="w-full"
							/>
							{isInviteFlow ? (
								<Button style={ButtonStyle.Transparent} onClick={() => setUsernameLocked((current) => !current)}>
									{usernameLocked ? "Change" : "Lock"}
								</Button>
							) : null}
						</div>
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
