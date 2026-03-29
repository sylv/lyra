import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { SessionList, SessionsQuery } from "../components/settings/sessions";

export const Route = createFileRoute("/settings/sessions")({
	component: RouteComponent,
});

function RouteComponent() {
	const { data } = useQuery(SessionsQuery, {
		pollInterval: 5_000,
	});

	return (
		<section className="space-y-4">
			<div>
				<h3>Sessions</h3>
				<p className="mt-1 text-sm text-zinc-400">View active watch sessions and the players currently in each one</p>
			</div>
			<SessionList sessions={data?.watchSessions ?? []} />
		</section>
	);
}
