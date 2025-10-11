import React, { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { BeatMap } from "@/schema";
import { R2_PUBLIC_URL, API_BASE } from "@/lib/api";
import { sanitizeText } from "@/lib/sanitize";

export type TransitionPhase = "forward" | "reverse";

interface TransitionOverlayProps {
  map: BeatMap;
  fromCardRect: DOMRect;
  fromTextRect: DOMRect | null;
  phase: TransitionPhase;
  onDone: () => void;
}

function computeBannerRect(): { width: number; height: number } {
  const temp = document.createElement("div");
  temp.style.position = "absolute";
  temp.style.top = "0";
  temp.style.left = "0";
  temp.style.right = "0";
  temp.style.height = "calc(100vw * 9 / 32)";
  temp.style.maxHeight = "320px";
  temp.style.visibility = "hidden";
  temp.style.pointerEvents = "none";
  document.body.appendChild(temp);
  const r = temp.getBoundingClientRect();
  document.body.removeChild(temp);
  return { width: r.width, height: r.height };
}

export default function TransitionOverlay({ map, fromCardRect, fromTextRect, phase, onDone }: TransitionOverlayProps) {
  const cardRef = useRef<HTMLDivElement | null>(null);
  const textRef = useRef<HTMLDivElement | null>(null);
  const cleanupTimer = useRef<number | null>(null);

  const safeSong = sanitizeText(map.song, 200);
  const safeArtist = sanitizeText(map.artist, 200);
  const safeCharter = sanitizeText(map.charter, 200);
  const baseImg = R2_PUBLIC_URL || API_BASE || '';
  const imgSrc = map.image ? `${baseImg}/thumbs/${map.id}.png` : (require("../public/beatblocks.jpg") as string);

  useEffect(() => {
    const cardEl = cardRef.current;
    const textEl = textRef.current;
    if (!cardEl) return;

    const start = () => {
      const { width: bannerW, height: bannerH } = computeBannerRect();
      
      cardEl.style.position = 'fixed';
      cardEl.style.transformOrigin = 'top left';
      cardEl.style.transition = 'transform 400ms ease-out';
      cardEl.style.zIndex = '100';

      if (phase === 'forward') {
        // Forward: start at card position, animate to banner
        cardEl.style.top = `${fromCardRect.top}px`;
        cardEl.style.left = `${fromCardRect.left}px`;
        cardEl.style.width = `${fromCardRect.width}px`;
        cardEl.style.height = `${fromCardRect.height}px`;
        cardEl.style.transform = 'none';

        if (textEl) {
          const tr = fromTextRect || fromCardRect;
          textEl.style.position = 'fixed';
          textEl.style.top = `${tr.top}px`;
          textEl.style.left = `${tr.left}px`;
          textEl.style.width = `${tr.width}px`;
          textEl.style.height = `${tr.height}px`;
          textEl.style.transition = 'transform 400ms ease-out, width 400ms ease-out';
          textEl.style.zIndex = '101';
          textEl.style.pointerEvents = 'none';
          textEl.style.transform = 'none';
        }

        requestAnimationFrame(() => {
          const scaleX = bannerW / fromCardRect.width;
          const scaleY = bannerH / fromCardRect.height;
          const tx = -fromCardRect.left;
          const ty = -fromCardRect.top;
          cardEl.style.transform = `translate(${tx}px, ${ty}px) scale(${scaleX}, ${scaleY})`;

          if (textEl) {
            const tr = fromTextRect || fromCardRect;
            const targetTop = (bannerH - tr.height) / 2;
            const dx = -tr.left;  // Move from card position to left edge (0)
            const dy = targetTop - tr.top;
            textEl.style.transform = `translate(${dx}px, ${dy}px)`;
            textEl.style.width = `${bannerW}px`;
          }
        });
      } else {
        // Reverse: start at banner position, animate to card
        cardEl.style.top = '0px';
        cardEl.style.left = '0px';
        cardEl.style.width = `${bannerW}px`;
        cardEl.style.height = `${bannerH}px`;
        cardEl.style.transform = 'none';

        if (textEl) {
          const tr = fromTextRect || fromCardRect;
          const targetTop = (bannerH - tr.height) / 2;
          textEl.style.position = 'fixed';
          textEl.style.top = `${targetTop}px`;
          textEl.style.left = '0px';
          textEl.style.width = `${bannerW}px`;
          textEl.style.height = `${tr.height}px`;
          textEl.style.transition = 'transform 400ms ease-out, width 400ms ease-out';
          textEl.style.zIndex = '101';
          textEl.style.pointerEvents = 'none';
          textEl.style.transform = 'none';
        }

        requestAnimationFrame(() => {
          const scaleX = fromCardRect.width / bannerW;
          const scaleY = fromCardRect.height / bannerH;
          const tx = fromCardRect.left;
          const ty = fromCardRect.top;
          cardEl.style.transform = `translate(${tx}px, ${ty}px) scale(${scaleX}, ${scaleY})`;

          if (textEl) {
            const tr = fromTextRect || fromCardRect;
            const currentTop = parseFloat(textEl.style.top || '0');
            const dx = tr.left;
            const dy = tr.top - currentTop;
            textEl.style.transform = `translate(${dx}px, ${dy}px)`;
            textEl.style.width = `${tr.width}px`;
          }
        });
      }

      // Safety cleanup after transition ends
      if (cleanupTimer.current) window.clearTimeout(cleanupTimer.current);
      cleanupTimer.current = window.setTimeout(() => onDone(), 450);
    };

    start();

    const onResize = () => {
      if (phase !== 'forward') return; // only need during forward (banner open)
      const { width: bannerW, height: bannerH } = computeBannerRect();
      const scaleX = bannerW / fromCardRect.width;
      const scaleY = bannerH / fromCardRect.height;
      const tx = -fromCardRect.left;
      const ty = -fromCardRect.top;
      cardEl.style.transform = `translate(${tx}px, ${ty}px) scale(${scaleX}, ${scaleY})`;
      if (textEl) {
        const pad = 24;
        const tr = fromTextRect || fromCardRect;
        const targetLeft = 0 + pad;
        const targetTop = (bannerH - tr.height) / 2;
        const dx = targetLeft - tr.left;
        const dy = targetTop - tr.top;
        const targetWidth = Math.max(0, bannerW - pad * 2);
        textEl.style.transform = `translate(${dx}px, ${dy}px)`;
        textEl.style.width = `${targetWidth}px`;
      }
    };

    window.addEventListener('resize', onResize);
    return () => {
      window.removeEventListener('resize', onResize);
      if (cleanupTimer.current) window.clearTimeout(cleanupTimer.current);
    };
  }, [phase, fromCardRect, fromTextRect, onDone]);

  const overlay = (
    <>
      <div ref={cardRef} id="animated-banner-card" style={{ pointerEvents: 'none' }}>
        <div className="absolute inset-0">
          <img src={imgSrc} alt={safeSong} className="w-full h-full object-cover" />
        </div>
        <div className="absolute inset-0 card-overlay-gradient" />
      </div>
      <div ref={textRef} className="font-['Press_Start_2P'] text-white p-6" style={{ overflow: 'hidden' }}>
        <div className="flex-1 min-w-0">
          <h2 className="text-xl mb-1 line-clamp-1">{safeSong}</h2>
          <p className="text-sm">by {safeArtist}</p>
          <p className="text-sm">Charter: {safeCharter}</p>
        </div>
      </div>
    </>
  );

  return createPortal(overlay, document.body);
}
