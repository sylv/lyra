import { CircleAlert } from "lucide-react";
import type { FC, ReactNode } from "react";
import { ErrorBoundary as ReactErrorBoundary } from "react-error-boundary";
import { useLocation } from "react-router";
import { cn } from "../lib/utils";
import { IconText } from "./icon-text";

type ErrorFallbackProps = {
  error: unknown;
  className?: string;
};

const ErrorFallback: FC<ErrorFallbackProps> = ({ error, className }) => {
  const message = error instanceof Error ? error.message : String(error);
  const text = message ? `Error: ${message}` : "Something went wrong";

  return (
    <div className={cn("h-full w-full flex items-center justify-center", className)}>
      <IconText icon={<CircleAlert className="size-4" />} text={text} />
    </div>
  );
};

type AppErrorBoundaryProps = {
  children: ReactNode;
  className?: string;
  resetKeys?: unknown[];
};

export const AppErrorBoundary: FC<AppErrorBoundaryProps> = ({ children, className, resetKeys = [] }) => {
  const location = useLocation();
  const { pathname, search, hash } = location;

  return (
    <ReactErrorBoundary
      resetKeys={[pathname, search, hash, ...resetKeys]}
      FallbackComponent={({ error }) => <ErrorFallback error={error} className={className} />}
    >
      {children}
    </ReactErrorBoundary>
  );
};
