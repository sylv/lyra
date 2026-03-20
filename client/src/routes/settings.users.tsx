import { useQuery } from "@apollo/client/react";
import { createFileRoute } from "@tanstack/react-router";
import { UserManager, UsersManagementQuery } from "../components/user-manager";

export const Route = createFileRoute("/settings/users")({
	component: RouteComponent,
});

function RouteComponent() {
	const { data, loading, error } = useQuery(UsersManagementQuery);

	return (
		<section className="space-y-4">
			<UserManager
				users={data?.users ?? []}
				viewerId={data?.viewer?.id ?? null}
				loading={loading}
				error={error?.message ?? null}
			/>
		</section>
	);
}
