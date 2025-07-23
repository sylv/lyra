/** biome-ignore-all lint/suspicious/noExplicitAny: its cringe */
import { useState, type FC } from "react";
import { Input } from "../../input";
import { SetupModalStep } from "../setup-modal-step";

interface LoginFormProps {
	refetch: () => Promise<void>;
}

export const LoginForm: FC<LoginFormProps> = ({ refetch }) => {
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

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
		<SetupModalStep loading={loading} disabled={loading} onSubmit={handleSubmit} error={error}>
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
		</SetupModalStep>
	);
};
