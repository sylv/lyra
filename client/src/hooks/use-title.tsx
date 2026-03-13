import { useEffect } from "react"

const defaultTitle = "Home"
const defaultSuffix = " — Lyra"

const resetTitle = () => {
    document.title = defaultTitle + defaultSuffix
}

export const useTitle = (title?: string) => {
    useEffect(() => {
        if (!title) {
            resetTitle()
        } else {
            document.title = title + defaultSuffix
            return resetTitle
        }
    }, [title])
}