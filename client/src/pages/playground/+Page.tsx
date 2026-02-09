import { DirectoryPicker } from "@/components/directory-picker"
import { useState, type FC } from "react";

export const Page: FC = () => {
    const [path, setPath] = useState<string | null>("/");

    return (
        <div className="p-6">
            <DirectoryPicker onPathChange={setPath} />
        </div>
    )
}