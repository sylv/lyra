import { useSuspenseQuery } from "../hooks/use-suspense-query";
import { UserManager, UsersManagementQuery } from "../components/settings/users";

export function SettingsUsersRoute() {
  const [{ data }] = useSuspenseQuery({ query: UsersManagementQuery });

  return (
    <section className="space-y-4">
      <div>
        <h3>Users</h3>
        <p className="mt-1 text-sm text-zinc-400">
          Create and manage user accounts, set permissions, and invite others
        </p>
      </div>
      <UserManager users={data.users} libraries={data.libraries} viewerId={data.viewer?.id ?? null} error={null} />
    </section>
  );
}
