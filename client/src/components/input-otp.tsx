import { OTPInput, REGEXP_ONLY_DIGITS, type SlotProps } from "input-otp";
import { Fragment, type FC } from "react";
import { cn } from "../lib/utils";

interface InputOtpProps {
	onChange: (code: number | null) => void;
}

export const InputOtp: FC<InputOtpProps> = ({ onChange }) => (
	<OTPInput
		maxLength={6}
		containerClassName="group flex items-center justify-center gap-2 has-[:disabled]:opacity-30 no-scrollbar"
		onChange={(code) => {
			if (code.length === 6) {
				onChange(Number(code));
			} else {
				onChange(null);
			}
		}}
		pattern={REGEXP_ONLY_DIGITS}
		pasteTransformer={(text) => text.replace(/[^0-9]/g, "")}
		render={({ slots }) => (
			<Fragment>
				<div className="flex gap-2">
					{slots.slice(0, 3).map((slot, idx) => (
						<Slot key={idx} {...slot} />
					))}
				</div>

				<FakeDash />

				<div className="flex gap-2">
					{slots.slice(3).map((slot, idx) => (
						<Slot key={idx} {...slot} />
					))}
				</div>
			</Fragment>
		)}
	/>
);

const Slot: FC<SlotProps> = (props) => {
	return (
		<div
			className={cn(
				"relative flex h-10 w-10 items-center justify-center rounded-sm text-sm",
				"bg-zinc-950 outline-none transition-colors",
				{ "bg-zinc-900": props.isActive },
			)}
		>
			<div className="group-has-[input[data-input-otp-placeholder-shown]]:text-accent-foreground/40">
				{props.char ?? props.placeholderChar}
			</div>
			{props.hasFakeCaret && <FakeCaret />}
		</div>
	);
};

const FakeCaret: FC = () => {
	return (
		<div className="absolute pointer-events-none inset-0 flex items-center justify-center animate-caret-blink">
			<div className="h-5 w-px bg-white" />
		</div>
	);
};

const FakeDash: FC = () => {
	return (
		<div className="flex w-4 items-center justify-center">
			<div className="h-px w-2 rounded-full bg-zinc-700" />
		</div>
	);
};
