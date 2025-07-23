import { Dialog, DialogContent, DialogOverlay, DialogPortal } from "@radix-ui/react-dialog";
import { useMemo, useState, type FC } from "react";
import { DynamicBackground } from "../dynamic-background";
import { CreateAccountForm } from "./steps/create-account-form";
import { LoginForm } from "./steps/login-form";
import { useApolloClient } from "@apollo/client";

export type InitState = { state: "login" } | { state: "create_first_user"; setup_token: string } | { state: "ready" };

interface SetupModalProps {
	state: InitState;
	mutate: () => Promise<InitState | undefined>;
	onClose: () => void;
}

export const SetupModal: FC<SetupModalProps> = ({ state, mutate, onClose }) => {
	const client = useApolloClient();
	const [stepId, setStepId] = useState<string | null>(null);
	const stepDescription = useMemo(() => {
		switch (state.state) {
			case "login":
				setStepId("login");
				return "Login to your account";
			case "create_first_user":
				setStepId("create_first_user");
				return "Create your first account";
			case "ready":
				setStepId(null);
				return "You're all set!";
		}
	}, [state]);

	const handleRefetch = async () => {
		const nextState = await mutate();
		if (!nextState) return;
		if (nextState.state === "ready") {
			// todo: if the current state is not "login", keep the modal open
			// and show some post-setup info (like where to find settings)
			client.refetchQueries({ include: "all" });
			onClose();
		}
	};

	return (
		<Dialog open={true}>
			<DialogPortal>
				<DialogOverlay className="fixed inset-0 bg-black/50 backdrop-blur-xs z-10" />
				<DialogContent className="z-20 rear outline-none fixed left-1/2 top-1/2 max-h-[85vh] w-[900px] h-[600px] max-w-[80vw] -translate-x-1/2 -translate-y-1/2 rounded-md bg-zinc-900 overflow-hidden">
					<div className="p-6 h-full flex flex-col">
						<h1 className="text-2xl font-bold">Let's get you sorted</h1>
						<p className="text-zinc-400 text-sm mb-6">{stepDescription}</p>
						{stepId === "create_first_user" && (
							<CreateAccountForm
								refetch={async () => {
									await mutate();
								}}
							/>
						)}
						{stepId === "login" && <LoginForm refetch={handleRefetch} />}
					</div>
					<div className="fixed inset-0 pointer-events-none">
						<DynamicBackground />
					</div>
				</DialogContent>
			</DialogPortal>
		</Dialog>
	);
};
