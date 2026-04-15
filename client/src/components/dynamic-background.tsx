import { useEffect, useRef, useState, type FC } from "react";
import { useStore } from "zustand/react";
// import defaultDynamicBackground from "../assets/default-dynamic-background.svg";
import { backgroundStore } from "../hooks/use-background";
import { getAssetImageUrl } from "./image";
import { generateGradientIcon } from "../lib/generate-gradient-icon";

const DURATION = 2000;
const generateDefault = () => generateGradientIcon(Date.now().toString(), { size: 512 });

// todo: we should extract primary colours and use those instead. loading full images for this is kinda crazy.
export const DynamicBackground: FC = () => {
	const backgroundAsset = useStore(backgroundStore);
	const backgroundUrl = backgroundAsset ? getAssetImageUrl(backgroundAsset, 200) : null;
	const [defaultBackground, setDefaultBackground] = useState<string>(generateDefault);
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
			if (backgroundUrl === current) {
				setShowCurrent(true);
			} else {
				setCurrent(backgroundUrl);
			}
		}, DURATION);
	}, [backgroundUrl]);

	const generateNewDefault = () => {
		setDefaultBackground(generateDefault());
	};

	return (
		<div
			id="dynamic-background"
			className="opacity-10 blur-3xl scale-[1.1] fixed inset-0 pointer-events-none select-none -z-10"
			aria-hidden
		>
			<img src={defaultBackground} alt="" className="absolute w-full h-full inset-0 object-fill" />
			{current && (
				<img
					src={current}
					alt=""
					decoding="async"
					className="absolute h-full w-full transition-opacity ease-in-out object-fill"
					style={{
						transitionDuration: `${DURATION}ms`,
						opacity: showCurrent ? 1 : 0,
					}}
					onLoad={(event) => {
						const image = event.target as HTMLImageElement;
						image.decode().then(() => {
							setShowCurrent(true);
							setTimeout(generateNewDefault, DURATION);
						});
					}}
				/>
			)}
		</div>
	);
};
