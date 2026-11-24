/**
 * Sidebar icons in the style of Windows 11 Settings: thin Fluent-style line
 * glyphs in the text colour, with the accent on the selected section. Static
 * markup, so it lives outside the components that use it.
 */

import type { ReactNode } from "react";

export type SectionId = "appearance" | "dock" | "windows" | "menubar" | "layout" | "system";

const GLYPHS: Record<SectionId, ReactNode> = {
  // A brush over a palette: personalization.
  appearance: (
    <>
      <path d="M12 3.5a8.5 8.5 0 1 0 0 17c1.2 0 1.8-.8 1.8-1.7 0-1.3-1.2-1.6-1.2-2.8 0-1 .8-1.7 1.8-1.7h2.1a4 4 0 0 0 4-4C20.5 6.6 16.7 3.5 12 3.5Z" />
      <circle cx="7.8" cy="11" r="1.1" />
      <circle cx="10.2" cy="7.4" r="1.1" />
      <circle cx="14.6" cy="7.6" r="1.1" />
    </>
  ),
  // A screen with a row of icons along its bottom.
  dock: (
    <>
      <rect x="3.5" y="4.5" width="17" height="15" rx="2.5" />
      <path d="M7.5 16.5h9" />
      <path d="M8 13.5h1.5M11.25 13.5h1.5M14.5 13.5H16" />
    </>
  ),
  // Two stacked windows.
  windows: (
    <>
      <rect x="7.5" y="7.5" width="13" height="11" rx="2" />
      <path d="M16.5 7.5V6a2 2 0 0 0-2-2h-9a2 2 0 0 0-2 2v7.5a2 2 0 0 0 2 2h2" />
      <path d="M7.5 10.5h13" />
    </>
  ),
  // A screen with a band across its top.
  menubar: (
    <>
      <rect x="3.5" y="4.5" width="17" height="15" rx="2.5" />
      <path d="M3.5 8.5h17" />
      <path d="M15.5 6.5h2.5" />
    </>
  ),
  // Three sliders: arranging and tuning.
  layout: (
    <>
      <path d="M4 7h16M4 12h16M4 17h16" />
      <circle cx="9" cy="7" r="1.9" className="icon-knob" />
      <circle cx="15" cy="12" r="1.9" className="icon-knob" />
      <circle cx="11" cy="17" r="1.9" className="icon-knob" />
    </>
  ),
  // A gear.
  system: (
    <>
      <circle cx="12" cy="12" r="2.8" />
      <path d="M10.6 3.8h2.8l.5 2.3 1.6.9 2.2-.8 1.4 2.4-1.8 1.6v1.6l1.8 1.6-1.4 2.4-2.2-.8-1.6.9-.5 2.3h-2.8l-.5-2.3-1.6-.9-2.2.8-1.4-2.4 1.8-1.6v-1.6L5 8.6l1.4-2.4 2.2.8 1.6-.9z" />
    </>
  ),
};

export function SectionIcon({ section }: { section: SectionId }) {
  return (
    <svg
      className="nav-icon"
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {GLYPHS[section]}
    </svg>
  );
}

/** The window's caption buttons, drawn as Windows 11 draws them. */
export function CaptionGlyph({ kind }: { kind: "minimize" | "maximize" | "close" }) {
  return (
    <svg viewBox="0 0 10 10" width="10" height="10" fill="none" stroke="currentColor" strokeWidth="1" aria-hidden="true">
      {kind === "minimize" ? <path d="M0 5.5h10" /> : null}
      {kind === "maximize" ? <rect x="0.5" y="0.5" width="9" height="9" rx="1.5" /> : null}
      {kind === "close" ? <path d="m0.5 0.5 9 9m0-9-9 9" /> : null}
    </svg>
  );
}
