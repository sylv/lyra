import { BookmarkCheckIcon, BookmarkIcon } from "lucide-react";
import { toast } from "sonner";
import { useMutation } from "urql";
import { graphql } from "../@generated/gql";
import { Button, ButtonSize, ButtonStyle } from "./button";
import { DropdownMenuItem } from "./ui/dropdown-menu";
import type { FC } from "react";

const AddNodeToWatchlistMutation = graphql(`
  mutation AddNodeToWatchlist($nodeId: String!) {
    addNodeToWatchlist(nodeId: $nodeId)
  }
`);

const RemoveNodeFromWatchlistMutation = graphql(`
  mutation RemoveNodeFromWatchlist($nodeId: String!) {
    removeNodeFromWatchlist(nodeId: $nodeId)
  }
`);

interface WatchlistControlProps {
  nodeId: string;
  inWatchlist: boolean;
}

const useWatchlistAction = ({ nodeId, inWatchlist }: WatchlistControlProps) => {
  const [{ fetching: adding }, addNodeToWatchlist] = useMutation(AddNodeToWatchlistMutation);
  const [{ fetching: removing }, removeNodeFromWatchlist] = useMutation(RemoveNodeFromWatchlistMutation);
  const fetching = adding || removing;
  const label = inWatchlist ? "Remove from Watchlist" : "Add to Watchlist";
  const Icon = inWatchlist ? BookmarkCheckIcon : BookmarkIcon;

  const toggleWatchlist = async () => {
    const result = inWatchlist ? await removeNodeFromWatchlist({ nodeId }) : await addNodeToWatchlist({ nodeId });

    if (result.error) {
      toast.error(result.error.message);
    }
  };

  return { fetching, label, Icon, toggleWatchlist };
};

export const WatchlistButton: FC<WatchlistControlProps> = (props) => {
  const { fetching, label, Icon, toggleWatchlist } = useWatchlistAction(props);

  return (
    <Button
      style={ButtonStyle.Glass}
      size={ButtonSize.Smol}
      className="w-fit"
      icon={["watchlist", Icon]}
      iconSide="left"
      loading={fetching}
      onClick={() => void toggleWatchlist()}
    >
      {label}
    </Button>
  );
};

export const WatchlistMenuItem: FC<WatchlistControlProps> = (props) => {
  const { fetching, label, Icon, toggleWatchlist } = useWatchlistAction(props);

  return (
    <DropdownMenuItem className="py-2" disabled={fetching} onSelect={() => void toggleWatchlist()}>
      <Icon className="size-4" />
      {label}
    </DropdownMenuItem>
  );
};
