import { usePlayerRuntimeStore } from "../player-runtime-store";

const toRuntimeSeconds = (runtimeMinutes: number | null | undefined) => {
  if (typeof runtimeMinutes !== "number" || !Number.isFinite(runtimeMinutes) || runtimeMinutes <= 0) return 0;
  return runtimeMinutes * 60;
};

export const usePlayerDisplayState = (
  media:
    | {
        defaultFile?: { probe?: { runtimeMinutes?: number | null } | null } | null;
        watchProgress?: { progressPercent: number; completed?: boolean | null } | null;
      }
    | null,
) => {
  const currentTime = usePlayerRuntimeStore((state) => state.currentTime);
  const duration = usePlayerRuntimeStore((state) => state.duration);
  const hasMediaLoaded = usePlayerRuntimeStore((state) => state.hasMediaLoaded);
  const targetTime = usePlayerRuntimeStore((state) => state.targetTime);

  const fallbackDuration = toRuntimeSeconds(media?.defaultFile?.probe?.runtimeMinutes);
  const fallbackProgress =
    media?.watchProgress && !media.watchProgress.completed && fallbackDuration > 0
      ? media.watchProgress.progressPercent * fallbackDuration
      : 0;

  return {
    currentTime: hasMediaLoaded ? currentTime : (targetTime ?? fallbackProgress),
    duration: hasMediaLoaded ? duration : fallbackDuration,
  };
};
