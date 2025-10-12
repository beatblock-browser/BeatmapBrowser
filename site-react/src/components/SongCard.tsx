import React from "react";
import { BeatMap } from "@/schema";
import { R2_PUBLIC_URL, API_BASE } from "@/lib/api";
import { sanitizeText } from "@/lib/sanitize";
// @ts-ignore
import default_image from "../public/beatblocks.jpg";

type Props = {
  map: BeatMap;
  className: string;
  onClick?: () => void;
  onKeyDown?: (e: React.KeyboardEvent) => void;
  onMouseEnter?: () => void;
  jwt?: string | null;
  hideActions?: boolean;
  imageOverrideUrl?: string | null;
  clickable?: boolean;
};

export default function SongCard({
  map,
  className,
  onClick,
  onKeyDown,
  onMouseEnter,
  jwt,
  hideActions = false,
  imageOverrideUrl = null,
  clickable = true,
}: Props) {
  const baseThumb = R2_PUBLIC_URL || API_BASE || '';
  const baseZip = R2_PUBLIC_URL || API_BASE || '';

  const imageSrc = imageOverrideUrl
    ? imageOverrideUrl
    : map.image
    ? `${baseThumb}/thumbs/${map.id}.png`
    : (default_image as string);

  const safeSong = sanitizeText(map.song, 200);
  const safeArtist = sanitizeText(map.artist, 200);
  const safeCharter = sanitizeText(map.charter, 200);

  // Map numeric difficulties to preset buckets
  const DIFFICULTY_ORDER = ['Apocraphyia','Challenge','Hard','Easy','Special'] as const;
  const bucketOf = (n: number): string => {
    if (typeof n !== 'number' || Number.isNaN(n)) return 'Special';
    if (n >= 1 && n < 2) return 'Easy';
    if (n >= 2 && n < 3) return 'Hard';
    if (n >= 3 && n < 4) return 'Challenge';
    if (n >= 4 && n < 5) return 'Apocraphyia';
    return 'Special';
  };
  const bucketSet = new Set<string>();
  (map.difficulties || []).forEach(d => bucketSet.add(bucketOf((d as any).difficulty)));
  const bucketList = DIFFICULTY_ORDER.filter(b => bucketSet.has(b));

  return (
    <div
      onClick={clickable ? onClick : undefined}
      onKeyDown={clickable ? onKeyDown : undefined}
      onMouseEnter={onMouseEnter}
      role={clickable ? "button" : undefined}
      tabIndex={clickable ? 0 : -1}
      className={className}
    >
      <img src={imageSrc} alt={safeSong} className="absolute inset-0 w-full h-full object-cover" />
      <div className="absolute inset-0 card-overlay-gradient" />
      <div className="absolute inset-0 flex flex-row items-center font-['Press_Start_2P'] text-white p-6">
        <div className="flex-1 min-w-0">
          <h2 className="text-xl mb-1 line-clamp-1">{safeSong}</h2>
          <p className="text-sm">by {safeArtist}</p>
          <p className="text-sm">Charter: {safeCharter}</p>
          {bucketList.length > 0 && (
            <div data-difficulty className="mt-2 flex flex-wrap gap-2">
              {bucketList.slice(0, 3).map((name) => (
                <span key={name} className="px-2 py-0.5 text-[10px] font-['Press_Start_2P'] border border-white/60 bg-black/40 rounded-sm">
                  {name}
                </span>
              ))}
              {bucketList.length > 3 && (
                <span className="px-2 py-0.5 text-[10px] font-['Press_Start_2P'] border border-white/40 bg-black/20 rounded-sm">
                  +{bucketList.length - 3}
                </span>
              )}
            </div>
          )}
        </div>
        {!hideActions && (
          <div className="absolute bottom-4 right-4 flex items-center gap-2">
            <div className="group relative">
              <a
                href={`${baseZip}/maps/${map.id}.zip`}
                className="inline-flex items-center justify-center px-3 py-2 pixel-btn bg-blue-500 text-white rounded-sm hover:bg-blue-600"
                onClick={(e) => e.stopPropagation()}
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>
              </a>
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1 bg-black/90 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 whitespace-nowrap font-['Press_Start_2P'] pointer-events-none">
                Download Map
                <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-black/90 rotate-45"></div>
              </div>
            </div>
            {!!jwt && (
              <div className="group relative">
                <button
                  className="inline-flex items-center justify-center px-3 py-2 pixel-btn bg-green-500 text-white rounded-sm hover:bg-green-600"
                  onClick={(e) => {
                    e.stopPropagation();
                    // TODO: Implement one-click install
                  }}
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                </button>
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1 bg-black/90 text-white text-xs rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 whitespace-nowrap font-['Press_Start_2P'] pointer-events-none">
                  One-Click Install
                  <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2 h-2 bg-black/90 rotate-45"></div>
                </div>
              </div>
            )}
            <div className="pixel-panel bg-white/90 px-3 py-1 rounded-sm">
              <span className="text-sm text-black">↑ {map.upvotes}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
