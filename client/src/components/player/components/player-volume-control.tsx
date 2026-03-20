import { Volume1Icon, Volume2Icon, VolumeIcon, VolumeXIcon } from "lucide-react";
import { useState, type FC } from "react";
import * as Slider from "@radix-ui/react-slider";
import { cn } from "../../../lib/utils";
import { PlayerButton } from "./player-button";

interface PlayerVolumeControlProps {
	volume: number;
	isMuted: boolean;
	onVolumeChange: (volume: number) => void;
	onToggleMute: () => void;
	onInteractionStart: () => void;
	onInteractionEnd: () => void;
	onActivity: () => void;
}

export const PlayerVolumeControl: FC<PlayerVolumeControlProps> = ({
	volume,
	isMuted,
	onVolumeChange,
	onToggleMute,
	onInteractionStart,
	onInteractionEnd,
	onActivity,
}) => {
	const [showSlider, setShowSlider] = useState(false);

	const getVolumeIcon = () => {
		if (isMuted || volume === 0) return <VolumeXIcon className="size-5" />;
		if (volume < 0.33) return <VolumeIcon className="size-5" />;
		if (volume < 0.66) return <Volume1Icon className="size-5" />;
		return <Volume2Icon className="size-5" />;
	};

	const handleSliderChange = (value: number[]) => {
		const newVolume = value[0];
		onActivity();
		onVolumeChange(newVolume);
	};

	return (
		<div
			className="relative flex items-center"
			onMouseEnter={() => {
				setShowSlider(true);
				onActivity();
			}}
			onMouseLeave={() => setShowSlider(false)}
		>
			<PlayerButton
				aria-label={isMuted ? "Unmute" : "Mute"}
				onClick={(e) => {
					e.stopPropagation();
					onActivity();
					onToggleMute();
				}}
			>
				{getVolumeIcon()}
			</PlayerButton>
			<div
				className={cn(
					"absolute left-full flex items-center transition-all duration-200",
					showSlider ? "opacity-100 translate-x-0" : "opacity-0 -translate-x-2 pointer-events-none",
				)}
			>
				<div className="py-6 px-2 flex items-center">
					<Slider.Root
						className="relative flex items-center w-20 h-5 cursor-pointer"
						value={[isMuted ? 0 : volume]}
						max={1}
						step={0.05}
						onValueChange={handleSliderChange}
						onPointerDown={onInteractionStart}
						onPointerUp={onInteractionEnd}
						onPointerCancel={onInteractionEnd}
						onLostPointerCapture={onInteractionEnd}
					>
						<Slider.Track className="bg-white/20 relative grow rounded-full h-1">
							<Slider.Range className="absolute bg-white rounded-full h-full" />
						</Slider.Track>
						<Slider.Thumb className="block size-3 bg-white rounded-full hover:bg-white/90 focus:outline-none focus:ring-2 focus:ring-white/50" />
					</Slider.Root>
				</div>
			</div>
		</div>
	);
};
