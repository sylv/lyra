import { Outlet, createFileRoute, useLocation } from "@tanstack/react-router";
import { Modal, ModalBody } from "../components/modal";
import { useSetup } from "../components/setup/setup-wrapper";
import { useTitle } from "../hooks/use-title";

export const Route = createFileRoute("/setup")({
	component: SetupRoute,
});

function SetupRoute() {
	const pathname = useLocation({
		select: (location) => location.pathname,
	});
	const { state } = useSetup();

	useTitle("Setup");

	return (
		<Modal
			open={true}
			onOpenChange={() => {}}
			className="h-[600px] w-[900px] max-h-[85vh] max-w-[80vw]"
			style={{ aspectRatio: "auto" }}
		>
			<ModalBody className="flex h-full flex-col p-6">
				{state && pathname !== "/setup" ? (
					<Outlet />
				) : (
					<div className="flex grow items-center justify-center text-sm text-zinc-400">Loading setup...</div>
				)}
			</ModalBody>
		</Modal>
	);
}
