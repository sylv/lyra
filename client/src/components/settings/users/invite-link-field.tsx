import { Copy } from "lucide-react";
import { useState, type FC } from "react";
import { Button, ButtonStyle } from "../../button";
import { Input } from "../../input";

export const InviteLinkField: FC<{ inviteLink: string }> = ({ inviteLink }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(inviteLink);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div className="space-y-2">
      <div className="text-xs font-medium uppercase tracking-wide text-zinc-500">Invite link</div>
      <div className="flex items-center gap-2">
        <Input value={inviteLink} readOnly className="w-full bg-zinc-700/40" />
        <Button
          onClick={handleCopy}
          style={ButtonStyle.Primary}
          icon={["copy", Copy]}
          iconSide="left"
          className="px-3 text-zinc-200 hover:text-zinc-100"
        >
          {copied ? "Copied" : "Copy"}
        </Button>
      </div>
    </div>
  );
};
