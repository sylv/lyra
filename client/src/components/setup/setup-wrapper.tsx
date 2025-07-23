import { useEffect, useState, type FC, type ReactNode } from "react";
import { ErrorBoundary } from "react-error-boundary";
import useSWR from "swr";
import { navigate } from "vike/client/router";
import { SetupModal, type InitState } from "./setup-modal";

const fetcher = (url: string) => fetch(url).then((res) => res.json());

export const SetupWrapper: FC<{ children: ReactNode }> = ({ children }) => {
	const [refreshInterval, setRefreshInterval] = useState<number | undefined>(5000);
	const [showModal, setShowModal] = useState(false);
	const { data, mutate } = useSWR<InitState>("/api/init", fetcher, {
		suspense: true,
		refreshInterval: refreshInterval,
	});

	useEffect(() => {
		if (!data) return;
		switch (data.state) {
			case "ready":
				// we don't close the modal because the modal will do it on its own
				// if it doesn't want to show the user some additional information after setup
				setRefreshInterval(undefined);
				break;
			case "login":
				setRefreshInterval(10000);
				setShowModal(true);
				break;
			case "create_first_user":
				setRefreshInterval(5000);
				navigate("/");
				setShowModal(true);
				break;
		}
	}, [data]);

	// todo: this error boundary should probably be elsewhere or be generic and inserted
	// in a couple places (eg, here, the parent component, the player maybe)
	return (
		<ErrorBoundary
			FallbackComponent={(props) => (
				<div className="h-full w-full flex items-center justify-center">Error: {props.error.message}</div>
			)}
		>
			{children}
			{showModal && data && <SetupModal state={data} mutate={mutate} onClose={() => setShowModal(false)} />}
		</ErrorBoundary>
	);
};
