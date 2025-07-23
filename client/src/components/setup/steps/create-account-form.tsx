/** biome-ignore-all lint/suspicious/noExplicitAny: its cringe */
import { useMutation, gql } from "@apollo/client";
import { useState, type FC } from "react";
import { SetupModalStep } from "../setup-modal-step";
import { InputOtp } from "../../input-otp";
import { Input } from "../../input";
import { Button, ButtonStyle } from "../../button";

const SIGNUP_MUTATION = gql`
	mutation Signup($username: String!, $password: String!) {
		signup(username: $username, password: $password) {
			id
			username
		}
	}
`;

interface CreateAccountFormProps {
	refetch: () => Promise<void>;
}

export const CreateAccountForm: FC<CreateAccountFormProps> = ({ refetch }) => {
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	const [waitingForCode, setWaitingForCode] = useState(true);
	const [setupCode, setSetupCode] = useState<number | null>(null);

	const [signup] = useMutation(SIGNUP_MUTATION, {});

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

			await refetch();
			setLoading(false);
		} catch (error: any) {
			// todo: handle 401s
			setError(error.message);
		} finally {
			setLoading(false);
		}
	};

	return (
		<SetupModalStep
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
					<p className="text-zinc-500 text-xs mt-3 text-center">Enter the code logged to the console on startup.</p>
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
		</SetupModalStep>
	);
};
