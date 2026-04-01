import { useEffect } from "react";
import { useQuery } from "urql";
import { SessionList, SessionsQuery } from "../components/settings/sessions";

export function SettingsSessionsRoute() {
	const [result, reexecuteQuery] = useQuery({ query: SessionsQuery });
	const { data } = result;

	useEffect(() => {
		const interval = window.setInterval(() => {
			reexecuteQuery({ requestPolicy: "network-only" });
		}, 5_000);

		return () => {
			window.clearInterval(interval);
		};
	}, [reexecuteQuery]);

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
