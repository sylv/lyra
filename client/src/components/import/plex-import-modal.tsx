import { useApolloClient, useMutation } from "@apollo/client/react";
import { AlertCircle, ArrowRight, CheckCircle2, Server, TriangleAlert } from "lucide-react";
import { useCallback, useMemo, useState, type FC } from "react";
import { graphql } from "../../@generated/gql";
import type { RunImportWatchStatesMutation } from "../../@generated/gql/graphql";
import { Button, ButtonStyle } from "../button";
import { Spinner } from "../ui/spinner";
import {
	INITIAL_PLEX_IMPORT_STATE,
	type PlexImportCompatibility,
	type PlexImportState,
	getNextStepFromCompatibility,
	FETCHING_METADATA_TEXT,
} from "./plex-import-state";
import { authenticateWithPlexPopup, discoverPlexServers, fetchPlexWatchStateRows } from "./plex-auth-popup";
import { Modal, ModalBody, ModalHeader } from "../modal";

const RunImportWatchStates = graphql(`
	mutation RunImportWatchStates($input: ImportWatchStatesInput!) {
		importWatchStates(input: $input) {
			dryRun
			totalRows
			matchedRows
			unmatchedRows
			conflictRows
			willInsert
			willOverwrite
			imported
			skipped
			conflicts {
				rowIndex
				sourceItemId
				title
				itemId
				existingProgressPercent
				importedProgressPercent
				reason
			}
			unmatched {
				rowIndex
				sourceItemId
				title
				reason
				ambiguous
			}
		}
	}
`);

interface PlexImportModalProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

const toCompatibility = (result: RunImportWatchStatesMutation["importWatchStates"]): PlexImportCompatibility => ({
	matchedRows: result.matchedRows,
	unmatchedRows: result.unmatchedRows,
	conflictRows: result.conflictRows,
	willInsert: result.willInsert,
	willOverwrite: result.willOverwrite,
});

export const PlexImportModal: FC<PlexImportModalProps> = ({ open, onOpenChange }) => {
	const apolloClient = useApolloClient();
	const [state, setState] = useState<PlexImportState>(INITIAL_PLEX_IMPORT_STATE);
	const [dryRunResult, setDryRunResult] = useState<RunImportWatchStatesMutation["importWatchStates"] | null>(null);
	const [finalResult, setFinalResult] = useState<RunImportWatchStatesMutation["importWatchStates"] | null>(null);
	const [runImportWatchStates] = useMutation(RunImportWatchStates);

	const resetState = useCallback(() => {
		setState(INITIAL_PLEX_IMPORT_STATE);
		setDryRunResult(null);
		setFinalResult(null);
	}, []);

	const setError = useCallback((message: string) => {
		setState((previous) => ({
			...previous,
			step: "error",
			errorMessage: message,
		}));
	}, []);

	const runCompatibilityCheck = useCallback(
		async (rows: PlexImportState["rows"]) => {
			setState((previous) => ({
				...previous,
				step: "checking",
				errorMessage: null,
			}));

			const response = await runImportWatchStates({
				variables: {
					input: {
						dryRun: true,
						overwriteConflicts: false,
						rows,
					},
				},
			});

			const result = response.data?.importWatchStates;
			if (!result) {
				throw new Error("Dry run returned no result");
			}

			const compatibility = toCompatibility(result);
			setDryRunResult(result);
			setState((previous) => ({
				...previous,
				compatibility,
				step: getNextStepFromCompatibility(compatibility),
			}));
		},
		[runImportWatchStates],
	);

	const fetchRowsForServer = useCallback(
		async (server: PlexImportState["servers"][number]) => {
			setState((previous) => ({
				...previous,
				selectedServerId: server.id,
				step: "fetching",
				errorMessage: null,
			}));

			const rows = await fetchPlexWatchStateRows(server);
			if (rows.length === 0) {
				throw new Error("No watched Plex items were found on this server");
			}

			setState((previous) => ({
				...previous,
				rows,
				overwriteConflicts: false,
				compatibility: null,
			}));

			await runCompatibilityCheck(rows);
		},
		[runCompatibilityCheck],
	);

	const handleConnect = useCallback(async () => {
		setState((previous) => ({
			...previous,
			step: "authenticating",
			errorMessage: null,
		}));

		try {
			const accountToken = await authenticateWithPlexPopup();
			const servers = await discoverPlexServers(accountToken, window.location.protocol);
			if (servers.length === 0) {
				throw new Error("No compatible Plex Media Server connections were found");
			}

			if (servers.length === 1) {
				setState((previous) => ({
					...previous,
					accountToken,
					servers,
					selectedServerId: servers[0].id,
				}));
				await fetchRowsForServer(servers[0]);
				return;
			}

			setState((previous) => ({
				...previous,
				accountToken,
				servers,
				selectedServerId: servers[0].id,
				step: "selectServer",
			}));
		} catch (error) {
			setError(error instanceof Error ? error.message : "Plex authentication failed");
		}
	}, [fetchRowsForServer, setError]);

	const handleLoadServer = useCallback(async () => {
		if (!state.selectedServerId) {
			setError("Select a Plex server first");
			return;
		}
		const server = state.servers.find((candidate) => candidate.id === state.selectedServerId);
		if (!server) {
			setError("Selected Plex server was not found");
			return;
		}

		try {
			await fetchRowsForServer(server);
		} catch (error) {
			setError(error instanceof Error ? error.message : "Failed to fetch Plex metadata");
		}
	}, [fetchRowsForServer, setError, state.selectedServerId, state.servers]);

	const estimatedImportCount = useMemo(() => {
		if (!state.compatibility) {
			return 0;
		}
		if (state.overwriteConflicts) {
			return state.compatibility.willInsert + state.compatibility.conflictRows;
		}
		return state.compatibility.willInsert;
	}, [state.compatibility, state.overwriteConflicts]);

	const handleImport = useCallback(async () => {
		if (state.rows.length === 0) {
			setError("No Plex rows were prepared for import");
			return;
		}

		setState((previous) => ({
			...previous,
			step: "importing",
			errorMessage: null,
		}));

		try {
			const response = await runImportWatchStates({
				variables: {
					input: {
						dryRun: false,
						overwriteConflicts: state.overwriteConflicts,
						rows: state.rows,
					},
				},
			});
			const result = response.data?.importWatchStates;
			if (!result) {
				throw new Error("Import mutation returned no result");
			}

			await apolloClient.refetchQueries({ include: "active" });
			setFinalResult(result);
			setState((previous) => ({
				...previous,
				step: "complete",
				accountToken: null,
				servers: [],
				selectedServerId: null,
				importedCount: result.imported,
				skippedCount: result.skipped,
			}));
		} catch (error) {
			setError(error instanceof Error ? error.message : "Plex import failed");
		}
	}, [apolloClient, runImportWatchStates, setError, state.overwriteConflicts, state.rows]);

	const handleOpenChange = useCallback(
		(nextOpen: boolean) => {
			if (!nextOpen) {
				resetState();
			}
			onOpenChange(nextOpen);
		},
		[onOpenChange, resetState],
	);

	return (
		<Modal open={open} onOpenChange={handleOpenChange} size="50vh">
			<ModalHeader>Import from Plex</ModalHeader>
			<ModalBody>
				{state.step === "connect" && (
					<div className="space-y-3">
						<p className="text-sm text-zinc-300">Connect your Plex account to begin importing your watch progress</p>
						<Button onClick={handleConnect} icon={["arrow-right", ArrowRight]}>
							Connect to Plex
						</Button>
					</div>
				)}

				{state.step === "authenticating" && (
					<div className="flex items-center gap-3 text-sm">
						<Spinner className="size-5" />
						<span>Waiting for Plex auth...</span>
					</div>
				)}

				{state.step === "selectServer" && (
					<div className="space-y-4">
						<div className="text-sm text-zinc-300">
							Multiple Plex Media Servers were found. Choose one to import from.
						</div>
						<select
							className="w-full rounded border border-zinc-400/30 bg-transparent px-3 py-2 text-sm"
							value={state.selectedServerId ?? ""}
							onChange={(event) =>
								setState((previous) => ({
									...previous,
									selectedServerId: event.target.value,
								}))
							}
						>
							{state.servers.map((server) => (
								<option key={server.id} value={server.id} className="text-black">
									{server.name} ({server.protocol.toUpperCase()}) {server.isRelay ? "relay" : "direct"} -{" "}
									{server.baseUrl}
								</option>
							))}
						</select>
						<Button onClick={handleLoadServer} icon={["server", Server]}>
							Fetch Watch States
						</Button>
					</div>
				)}

				{state.step === "fetching" && (
					<div className="flex items-center gap-3 text-sm">
						<Spinner className="size-5" />
						<span>{FETCHING_METADATA_TEXT}</span>
					</div>
				)}

				{state.step === "checking" && (
					<div className="flex items-center gap-3 text-sm">
						<Spinner className="size-5" />
						<span>Checking compatibility...</span>
					</div>
				)}

				{state.step === "conflicts" && dryRunResult && (
					<div className="space-y-4">
						<div className="flex items-start gap-3 rounded border border-yellow-500/40 bg-yellow-500/10 p-3">
							<TriangleAlert className="mt-0.5 size-4 text-yellow-300" />
							<div className="text-sm">
								<p className="font-semibold text-yellow-100">{dryRunResult.conflictRows} conflicts detected</p>
								<p className="text-yellow-100/80">Existing Lyra watch progress differs for these items.</p>
							</div>
						</div>

						{dryRunResult.conflicts.length > 0 && (
							<div className="max-h-36 space-y-1 overflow-auto rounded border border-zinc-400/20 p-2 text-xs">
								{dryRunResult.conflicts.slice(0, 10).map((conflict) => (
									<div key={`${conflict.rowIndex}-${conflict.itemId}`} className="text-zinc-300">
										#{conflict.rowIndex + 1} {conflict.title ?? conflict.sourceItemId ?? conflict.itemId}:{" "}
										{Math.round(conflict.existingProgressPercent * 100)}
										{" -> "}
										{Math.round(conflict.importedProgressPercent * 100)}%
									</div>
								))}
							</div>
						)}

						<label className="flex items-center gap-2 text-sm text-zinc-200">
							<input
								type="checkbox"
								checked={state.overwriteConflicts}
								onChange={(event) =>
									setState((previous) => ({
										...previous,
										overwriteConflicts: event.target.checked,
									}))
								}
							/>
							Overwrite conflicting watch states
						</label>

						<Button
							onClick={() =>
								setState((previous) => ({
									...previous,
									step: "confirm",
								}))
							}
							icon={["arrow-right", ArrowRight]}
						>
							Continue
						</Button>
					</div>
				)}

				{state.step === "confirm" && state.compatibility && (
					<div className="space-y-4">
						<div className="text-sm text-zinc-200">
							Import {estimatedImportCount} items?
							<div className="mt-2 text-xs text-zinc-400">
								{state.compatibility.unmatchedRows} unmatched items
								{state.compatibility.conflictRows > 0 ? ` and ${state.compatibility.conflictRows} conflicts` : ""} will
								be skipped.
							</div>
						</div>
						<div className="flex items-center gap-2">
							<Button onClick={handleImport} icon={["arrow-right", ArrowRight]}>
								Import Now
							</Button>
							<Button
								style={ButtonStyle.Transparent}
								onClick={() =>
									setState((previous) => ({
										...previous,
										step: previous.compatibility?.conflictRows ? "conflicts" : "connect",
									}))
								}
							>
								Back
							</Button>
						</div>
					</div>
				)}

				{state.step === "importing" && (
					<div className="flex items-center gap-3 text-sm">
						<Spinner className="size-5" />
						<span>Importing watch states...</span>
					</div>
				)}

				{state.step === "complete" && finalResult && (
					<div className="space-y-3">
						<div className="flex items-center gap-2 text-emerald-300">
							<CheckCircle2 className="size-4" />
							<span className="text-sm font-semibold">Import complete</span>
						</div>
						<div className="text-sm text-zinc-200">
							Imported {state.importedCount} rows and skipped {state.skippedCount}.
						</div>
						<div className="text-xs text-zinc-400">
							Unmatched: {finalResult.unmatchedRows} | Conflicts: {finalResult.conflictRows}
						</div>
						<Button
							onClick={() => {
								resetState();
								onOpenChange(false);
							}}
						>
							Close
						</Button>
					</div>
				)}

				{state.step === "error" && (
					<div className="space-y-3">
						<div className="flex items-start gap-2 rounded border border-red-500/40 bg-red-500/10 p-3 text-sm text-red-200">
							<AlertCircle className="mt-0.5 size-4" />
							<span>{state.errorMessage ?? "An unknown error occurred"}</span>
						</div>
						<Button onClick={resetState}>Start Over</Button>
					</div>
				)}
			</ModalBody>
		</Modal>
	);
};
