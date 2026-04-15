import { useState } from "react";
import { useQuery } from "urql";
import { LibrariesQuery, LibraryManager } from "../libraries";
import { SetupStep } from "./setup-step";
import { useSetup } from "./setup-wrapper";

export function CreateLibraryStep() {
  const { recheckSetup, isRechecking } = useSetup();
  const [{ data: librariesData, fetching: loading }] = useQuery({ query: LibrariesQuery });
  const [error, setError] = useState<string | null>(null);
  const libraries = librariesData?.libraries || [];
  const submitting = loading || isRechecking;

  const handleSubmit = async () => {
    setError(null);

    try {
      await recheckSetup();
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : "Failed to check setup state");
    }
  };

  return (
    <SetupStep
      loading={submitting}
      disabled={libraries.length === 0 || submitting}
      onSubmit={() => {
        void handleSubmit();
      }}
      centered={false}
      error={error}
    >
      <LibraryManager libraries={libraries} loading={loading} className="mb-6" />
    </SetupStep>
  );
}
