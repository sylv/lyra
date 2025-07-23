/** biome-ignore-all lint/suspicious/noArrayIndexKey: its cringe */
import { OTPInput, REGEXP_ONLY_DIGITS, type SlotProps } from "input-otp";
import { Fragment, type FC } from "react";
import { cn } from "../lib/utils";

interface InputOtpProps {
	onChange: (code: number | null) => void;
}

export const InputOtp: FC<InputOtpProps> = ({ onChange }) => (
	<OTPInput
		maxLength={6}
		containerClassName="group flex items-center has-[:disabled]:opacity-30 no-scrollbar"
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
				<div className="flex">
					{slots.slice(0, 3).map((slot, idx) => (
						<Slot key={idx} {...slot} />
					))}
				</div>

				<FakeDash />

				<div className="flex">
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
				"relative w-10 h-14 text-[2rem]",
				"flex items-center justify-center",
				"border-border border-y border-r first:border-l first:rounded-l-md last:rounded-r-md",
				"group-hover:border-accent-foreground/20 group-focus-within:border-accent-foreground/20",
				"outline-0 outline-accent-foreground/20",
				{ "outline-1 outline-accent-foreground": props.isActive },
			)}
		>
			<div className="group-has-[input[data-input-otp-placeholder-shown]]:opacity-20">
				{props.char ?? props.placeholderChar}
			</div>
			{props.hasFakeCaret && <FakeCaret />}
		</div>
	);
};

const FakeCaret: FC = () => {
	return (
		<div className="absolute pointer-events-none inset-0 flex items-center justify-center animate-caret-blink">
			<div className="w-px h-8 bg-white" />
		</div>
	);
};

const FakeDash: FC = () => {
	return (
		<div className="flex w-10 justify-center items-center">
			<div className="w-3 h-1 rounded-full bg-border" />
		</div>
	);
};
