import { useEffect, useRef, useState, type FC } from "react";
import { useStore } from "zustand/react";
import defaultDynamicBackground from "../../assets/default-dynamic-background.svg";
import { backgroundStore } from "../hooks/use-background";

const DURATION = 2000;

// todo: once images are stored in the db, we should extract primary colours and use those instead.
// loading full images for this is kinda crazy.
export const DynamicBackground: FC = () => {
	const backgroundUrl = useStore(backgroundStore);
	const [current, setCurrent] = useState<string | null>(null);
	const [showCurrent, setShowCurrent] = useState(false);
	const isInitial = useRef(true);
	const timerRef = useRef<NodeJS.Timeout | null>(null);

	useEffect(() => {
		if (timerRef.current) {
			clearTimeout(timerRef.current);
		}

		if (isInitial.current) {
			// without this we would have to wait for the transition duration unnecessarily
			// on first load we want to load the image asap
			isInitial.current = false;
			setShowCurrent(false);
			setCurrent(backgroundUrl);
			return;
		}

		setShowCurrent(false);
		timerRef.current = setTimeout(() => {
			setCurrent(backgroundUrl);
		}, DURATION);
	}, [backgroundUrl]);

	return (
		<div className="fixed inset-0 pointer-events-none h-dvw w-dvw blur-3xl opacity-10">
			<img
				src={defaultDynamicBackground}
				aria-hidden
				className="fixed object-cover h-full w-full"
			/>
			{current && (
				<img
					src={current}
					aria-hidden
					decoding="async"
					className="fixed object-cover h-full w-full transition-opacity ease-in-out"
					style={{
						transitionDuration: `${DURATION}ms`,
						opacity: showCurrent ? 1 : 0,
					}}
					onLoad={(event) => {
						const image = event.target as HTMLImageElement;
						image.decode().then(() => {
							setShowCurrent(true);
						});
					}}
				/>
			)}
		</div>
	);
};
