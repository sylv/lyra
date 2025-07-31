import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { ChevronDown, type LucideIcon } from "lucide-react";
import React, { useState, type FC, type ReactNode } from "react";
import { cn } from "../lib/utils";
import { Skeleton } from "./skeleton";

interface FilterButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
	children: ReactNode;
	active?: boolean;
}

export const FilterButton: FC<FilterButtonProps> = ({ children, active = false, className, ...rest }) => {
	return (
		<button
			type="button"
			className={cn(
				`flex rounded-lg px-4 py-0.5 text-sm gap-3 items-center transition-colors border border-zinc-700/50 text-zinc-200 outline-none select-none`,
				active ? "bg-zinc-200/10" : "hover:bg-zinc-200/10",
				className,
			)}
			{...rest}
		>
			{children}
		</button>
	);
};

export const FilterButtonSkeleton: FC = () => {
	return <Skeleton className="h-6 w-16 rounded-lg" />;
};

interface FilterSelectOption<T> {
	label: string;
	icon?: LucideIcon;
	value: T;
}

interface FilterSelectProps<T> {
	label: string;
	options: FilterSelectOption<T>[];
	value: T;
	onValueChange: (value: T) => void;
}

export const FilterSelect = <T extends string | number>({ options, value, onValueChange }: FilterSelectProps<T>) => {
	const [changed, setChanged] = useState(false);
	const selectedOption = options.find((option) => option.value === value) || options[0];

	return (
		<DropdownMenu.Root>
			<DropdownMenu.Trigger asChild>
				<FilterButton active={changed}>
					{selectedOption.icon && <selectedOption.icon className="h-3.5 w-3.5 text-zinc-50" />}
					{selectedOption.label}
					<ChevronDown className="h-3 w-3 text-zinc-400" />
				</FilterButton>
			</DropdownMenu.Trigger>
			<DropdownMenu.Portal>
				<DropdownMenu.Content
					sideOffset={5}
					className={cn(
						"z-50 min-w-[8rem] overflow-hidden rounded-lg border border-zinc-700/50 backdrop-blur-2xl bg-black/50 glass p-1 shadow-lg space-y-1",
						"data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
						"data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95",
						"data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
					)}
				>
					{options.map((option) => (
						<DropdownMenu.Item
							key={option.value}
							className={cn(
								"relative flex cursor-default select-none items-center rounded-md px-3 py-1 text-sm outline-none transition-colors",
								"text-zinc-200 focus:bg-zinc-200/10",
								"data-[disabled]:pointer-events-none data-[disabled]:opacity-50",
								option.value === value ? "bg-zinc-400/10" : "hover:bg-zinc-500/10",
							)}
							onSelect={() => {
								setChanged(true);
								onValueChange(option.value);
							}}
						>
							{option.icon && <option.icon className="h-3.5 w-3.5 mr-2 text-zinc-500" />}
							{option.label}
						</DropdownMenu.Item>
					))}
				</DropdownMenu.Content>
			</DropdownMenu.Portal>
		</DropdownMenu.Root>
	);
};

export const FilterSelectSkeleton: FC = () => {
	return <Skeleton className="h-6 w-24 rounded-lg" />;
};
