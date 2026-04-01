import { useQuery } from "urql";
import { UserManager, UsersManagementQuery } from "../components/settings/users";

export function SettingsUsersRoute() {
	const [{ data, error }] = useQuery({ query: UsersManagementQuery, context: { suspense: true } });

	return (
		<section className="space-y-4">
			<div>
				<h3>Users</h3>
				<p className="mt-1 text-sm text-zinc-400">
					Create and manage user accounts, set permissions, and invite others
				</p>
			</div>
			<UserManager
				users={data?.users ?? []}
				libraries={data?.libraries ?? []}
				viewerId={data?.viewer?.id ?? null}
				error={error?.message ?? null}
			/>
		</section>
	);
}
