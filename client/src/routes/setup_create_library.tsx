import { SetupPage } from "../components/settings/setup/setup-page";
import { CreateLibraryStep } from "../components/settings/setup/create-library-step";
import { useSetup } from "../components/settings/setup/setup-wrapper";
import { useTitle } from "../hooks/use-title";

export function SetupCreateLibraryRoute() {
  const { state } = useSetup();

  useTitle("Create a library");

  if (state?.state !== "create_first_library") {
    return null;
  }

  return (
    <SetupPage title="Let's get you sorted" description="Set up your media libraries">
      <CreateLibraryStep />
    </SetupPage>
  );
}
