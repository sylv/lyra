import type { FC } from "react";
import { graphql } from "../@generated/gql";
import type { GetActivitiesQuery } from "../@generated/gql/graphql";

export const ActivityPanelQuery = graphql(`
	query GetActivities {
		activities {
			taskType
			title
			current
			total
			progressPercent
		}
	}
`);

const CIRCLE_RADIUS = 12;
const CIRCLE_CIRCUMFERENCE = 2 * Math.PI * CIRCLE_RADIUS;

const CircularProgress: FC<{ progress: number }> = ({ progress }) => {
	const clampedProgress = Math.max(0, Math.min(1, progress));
	const strokeDashoffset = CIRCLE_CIRCUMFERENCE * (1 - clampedProgress);

	return (
		<div className="relative h-10 w-10 shrink-0">
			<svg className="h-10 w-10 -rotate-90" viewBox="0 0 36 36" aria-hidden="true">
				<circle cx="18" cy="18" r={CIRCLE_RADIUS} className="fill-none stroke-zinc-700/80" strokeWidth="3" />
				<circle
					cx="18"
					cy="18"
					r={CIRCLE_RADIUS}
					className="fill-none stroke-zinc-200 transition-all duration-300"
					strokeWidth="3"
					strokeLinecap="round"
					strokeDasharray={CIRCLE_CIRCUMFERENCE}
					strokeDashoffset={strokeDashoffset}
				/>
			</svg>
		</div>
	);
};

export const ActivityPanel: FC<{ data?: GetActivitiesQuery; open: boolean }> = ({ data }) => {
	// todo: animations when activities enter/leave so the modal doesn't jump around as much
	return (
		<div className="w-95 max-h-[70vh] overflow-y-auto bg-black p-3 shadow-lg shadow-black/30 min-h-[30vh]">
			<h2 className="px-1 pt-1 text-xs font-semibold">Activity</h2>
			{!data && <p className="text-sm text-zinc-400">Loading activity...</p>}
			{data?.activities?.length === 0 && (
				<p className="text-xs text-zinc-400 px-1 mt-2">No activity is currently running.</p>
			)}
			<div className="mt-1 space-y-2">
				{data?.activities?.map((task) => {
					return (
						<div key={task.taskType} className="flex items-center gap-3 py-2">
							<CircularProgress progress={task.progressPercent} />
							<div className="flex-1">
								<p className="text-sm font-semibold text-zinc-100">{task.title}</p>
								<p className="text-[0.67rem] text-zinc-400">
									{task.current < task.total ? "Processing" : "Processed"} {task.current.toLocaleString()} of{" "}
									{task.total.toLocaleString()}
								</p>
							</div>
						</div>
					);
				})}
			</div>
		</div>
	);
};
