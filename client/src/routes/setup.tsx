import { Outlet, createFileRoute, useLocation } from "@tanstack/react-router";
import { Modal } from "../components/modal";
import { useSetup } from "../components/setup/setup-wrapper";

export const Route = createFileRoute("/setup")({
	component: SetupRoute,
});

function SetupRoute() {
	const { state } = useSetup();
	const pathname = useLocation({
		select: (location) => location.pathname,
	});

	return (
		<Modal
			open={true}
			onOpenChange={() => {}}
			className="h-[600px] w-[900px] max-h-[85vh] max-w-[80vw]"
			style={{ aspectRatio: "auto" }}
		>
			{state && pathname !== "/setup" ? (
				<Outlet />
			) : (
				<div className="flex grow items-center justify-center text-sm text-zinc-400">Loading setup...</div>
			)}
		</Modal>
	);
}
