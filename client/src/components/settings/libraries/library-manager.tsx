import { useState, type FC } from "react";
import type { FragmentType } from "../../../@generated/gql";
import { cn } from "../../../lib/utils";
import { ManagementCreateCard } from "../management-card";
import type { LibraryCardFragment as LibraryCardData } from "../../../@generated/gql/graphql";
import { LibraryCard, LibraryCardFragment } from "./library-card";
import { LibraryFormModal } from "./library-form-modal";

interface LibraryManagerProps {
  libraries: Array<{ id: string } & FragmentType<typeof LibraryCardFragment>>;
  loading?: boolean;
  className?: string;
}

export const LibraryManager: FC<LibraryManagerProps> = ({ libraries, loading = false, className }) => {
  const [activeForm, setActiveForm] = useState<
    | { mode: "create" }
    | {
        mode: "edit";
        library: LibraryCardData;
      }
    | null
  >(null);

  return (
    <div className={cn("space-y-4", className)}>
      {activeForm ? <LibraryFormModal activeForm={activeForm} onClose={() => setActiveForm(null)} /> : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {libraries.map((library) => (
          <LibraryCard
            key={library.id}
            library={library}
            onEdit={(nextLibrary) => setActiveForm({ mode: "edit", library: nextLibrary })}
          />
        ))}

        <ManagementCreateCard
          title="New Library"
          description="Create another scan root for movies, shows, or mixed media."
          onClick={() => setActiveForm({ mode: "create" })}
          loading={loading}
        />
      </div>

      {!loading && libraries.length === 0 ? (
        <p className="text-sm text-zinc-500">No libraries yet. Add one to start importing media.</p>
      ) : null}
    </div>
  );
};
