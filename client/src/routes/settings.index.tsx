import { Navigate, createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings/")({
	component: RouteComponent,
});

function RouteComponent() {
	return <Navigate to="/settings/about" replace />;
}
