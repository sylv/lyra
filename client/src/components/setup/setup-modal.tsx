import { useMemo, type FC } from "react";
import { Modal, ModalBody } from "../modal";
import { CreateAccountForm } from "./steps/create-account-form";
import { LoginForm } from "./steps/login-form";
import { CreateLibraryStep } from "./steps/create-library-step";

export type InitState =
	| { state: "login" }
	| { state: "create_first_user"; setup_token: string }
	| { state: "create_first_library" }
	| { state: "ready" };

interface SetupModalProps {
	state: InitState;
	mutate: () => Promise<void>;
}

export const SetupModal: FC<SetupModalProps> = ({ state, mutate }) => {
	const setupStep = useMemo(() => {
		switch (state.state) {
			case "login":
				return {
					stepId: "login",
					description: "Login to your account",
				};
			case "create_first_user":
				return {
					stepId: "create_first_user",
					description: "Create your first account",
				};
			case "create_first_library":
				return {
					stepId: "create_first_library",
					description: "Set up your media libraries",
				};
			case "ready":
				return {
					stepId: null,
					description: "You're all set!",
				};
		}
	}, [state]);

	return (
		<Modal
			open={true}
			onOpenChange={() => {}}
			className="h-[600px] w-[900px] max-h-[85vh] max-w-[80vw] bg-zinc-900"
			style={{ aspectRatio: "auto" }}
		>
			<ModalBody patterned={false} className="flex h-full flex-col p-6">
				<h1 className="text-2xl font-bold">Let's get you sorted</h1>
				<p className="mb-6 text-sm text-zinc-400">{setupStep.description}</p>
				{setupStep.stepId === "create_first_user" && (
					<CreateAccountForm
						refetch={async () => {
							await mutate();
						}}
					/>
				)}
				{setupStep.stepId === "create_first_library" && <CreateLibraryStep refetch={mutate} />}
				{setupStep.stepId === "login" && <LoginForm refetch={mutate} />}
			</ModalBody>
		</Modal>
	);
};
