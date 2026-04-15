import { ImageIcon } from "lucide-react";
import type { FC } from "react";
import { graphql, unmask, type FragmentType } from "../@generated/gql";
import type { ImageAssetFragment } from "../@generated/gql/graphql";
import { appendAssetQuery } from "../lib/assets";
import { getThumbhashDataUrl } from "../lib/thumbhash";
import { cn } from "../lib/utils";

export enum ImageType {
  Poster = "poster",
  Thumbnail = "thumbnail",
  Avatar = "avatar",
}

export const Fragment = graphql(`
  fragment ImageAsset on Asset {
    id
    signedUrl
    thumbhash
  }
`);

interface ImageTypeConfig {
  baseClasses: string;
  fallbackTextClasses: string;
  proxyWidth: number;
}

const IMAGE_TYPE_CONFIG: Record<ImageType, ImageTypeConfig> = {
  [ImageType.Poster]: {
    baseClasses: "aspect-[2/3] bg-zinc-700/30 shrink-0 select-none w-full",
    fallbackTextClasses:
      "max-w-[60%] text-sm text-center font-semibold whitespace-normal wrap-break-words wrap-anywhere",
    proxyWidth: 400,
  },
  [ImageType.Thumbnail]: {
    baseClasses: "bg-zinc-700/30 shrink-0 aspect-[16/9] object-cover select-none h-38",
    fallbackTextClasses: "text-sm text-center font-semibold",
    proxyWidth: 600,
  },
  [ImageType.Avatar]: {
    baseClasses: "rounded-full bg-zinc-700/30 shrink-0 select-none aspect-square object-cover",
    fallbackTextClasses: "text-sm text-center font-semibold",
    proxyWidth: 200,
  },
};

interface ImageProps {
  type: ImageType;
  asset: FragmentType<typeof Fragment> | null | undefined;
  alt: string;
  className?: string;
}

export const getAssetImageUrl = (asset: ImageAssetFragment, height: number): string => {
  return appendAssetQuery(asset.signedUrl, {
    height,
  });
};

export const Image: FC<ImageProps> = ({ type, asset, alt, className }) => {
  const config = IMAGE_TYPE_CONFIG[type];
  const resolvedAsset = asset ? unmask(Fragment, asset) : null;
  const thumbhashPreview = getThumbhashDataUrl(resolvedAsset?.thumbhash);

  if (!resolvedAsset) {
    return (
      <div
        className={cn(
          config.baseClasses,
          "flex flex-col justify-center items-center gap-2 text-zinc-500 p-4 overflow-hidden",
          className,
        )}
      >
        <ImageIcon />
        <span className={config.fallbackTextClasses}>{alt}</span>
      </div>
    );
  }

  return (
    <img
      src={getAssetImageUrl(resolvedAsset, config.proxyWidth)}
      alt={alt}
      className={cn(config.baseClasses, className)}
      style={
        thumbhashPreview
          ? {
              backgroundImage: `url(${thumbhashPreview})`,
              backgroundSize: "cover",
              backgroundPosition: "center",
              backgroundRepeat: "no-repeat",
            }
          : undefined
      }
    />
  );
};
