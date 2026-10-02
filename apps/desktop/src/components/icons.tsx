/**
 * Original icon set for the editor.
 *
 * Every shape here was drawn for this project. Nothing is traced or copied
 * from game artwork or from any console maker's interface.
 */

import type { ReactElement, SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement> & { size?: number };

function base({ size = 22, ...rest }: IconProps): SVGProps<SVGSVGElement> {
  return {
    width: size,
    height: size,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 2.2,
    strokeLinecap: "round",
    strokeLinejoin: "round",
    "aria-hidden": true,
    focusable: false,
    ...rest,
  };
}

/** A boxing glove. Used as the app mark. */
export function GloveIcon({ size = 22, ...rest }: IconProps): ReactElement {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden focusable={false} {...rest}>
      <path
        fill="currentColor"
        d="M9.2 2.5c-3 0-5.4 2.4-5.4 5.4v3.6c0 1.9 1 3.6 2.5 4.6v.9h10v-1.2c2.300-.9 3.900-3.100 3.900-5.700V9.700c0-1.700-1.400-3.100-3.100-3.100-.500 0-1 .100-1.400.300C15 4.300 12.900 2.500 10.400 2.500H9.200Z"
      />
      <path
        fill="none"
        stroke="var(--glove-line, rgba(0,0,0,0.28))"
        strokeWidth="1.5"
        strokeLinecap="round"
        d="M15.600 7.200c-.700.500-1.100 1.300-1.100 2.200v1.400c0 1-.800 1.800-1.800 1.800H9.500"
      />
      <rect x="5.500" y="18.200" width="11.600" height="3.600" rx="1.300" fill="currentColor" opacity="0.72" />
    </svg>
  );
}

export function BoxersIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <circle cx="9" cy="8" r="3.500" />
      <path d="M2.800 20c.400-3.600 2.900-6 6.200-6s5.800 2.400 6.200 6" />
      <circle cx="17.500" cy="9.500" r="2.500" />
      <path d="M17.800 14.600c2 .300 3.300 2 3.600 4.400" />
    </svg>
  );
}

export function PaintIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="M12 3a9 9 0 1 0 0 18c1.400 0 2-.900 2-1.900 0-1.300-1-1.700-1-2.900 0-1 .800-1.700 1.900-1.700H17a4 4 0 0 0 4-4C21 6.400 17 3 12 3Z" />
      <circle cx="7.800" cy="11" r="1.100" fill="currentColor" stroke="none" />
      <circle cx="11" cy="7.300" r="1.100" fill="currentColor" stroke="none" />
      <circle cx="15.600" cy="8" r="1.100" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function LookIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <circle cx="10.500" cy="10.500" r="6.500" />
      <path d="m15.500 15.500 5 5" />
    </svg>
  );
}

export function CompareIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <rect x="3" y="4" width="7.500" height="16" rx="2" />
      <rect x="13.500" y="4" width="7.500" height="16" rx="2" />
    </svg>
  );
}

export function PlayIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <rect x="2.500" y="7" width="19" height="11" rx="5.500" />
      <path d="M7 10.500v4M5 12.500h4" />
      <circle cx="15.200" cy="13.600" r="0.900" fill="currentColor" stroke="none" />
      <circle cx="18" cy="11.400" r="0.900" fill="currentColor" stroke="none" />
    </svg>
  );
}

export function SaveIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="M5 3h11l4 4v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2Z" />
      <path d="M7.500 3v5h7V3M7 21v-7h10v7" />
    </svg>
  );
}

export function GearIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <circle cx="12" cy="12" r="3" />
      <path d="M12 2.500v3M12 18.500v3M2.500 12h3M18.500 12h3M5.300 5.300l2.100 2.100M16.600 16.600l2.100 2.100M5.300 18.700l2.100-2.100M16.600 7.400l2.100-2.100" />
    </svg>
  );
}

export function HelpIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <circle cx="12" cy="12" r="9.500" />
      <path d="M9.200 9.200a2.900 2.900 0 1 1 4.300 2.500c-.900.500-1.500 1.100-1.500 2.100" />
      <circle cx="12" cy="17.200" r="0.600" fill="currentColor" />
    </svg>
  );
}

export function ToolsIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="M14.500 6.200a4 4 0 0 0-5.300 5.200L3.500 17.100a2 2 0 1 0 2.900 2.900l5.700-5.700a4 4 0 0 0 5.200-5.300l-2.500 2.500-2.300-.500-.500-2.300 2.500-2.500Z" />
    </svg>
  );
}

export function UndoIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="M8 5 3.500 9.500 8 14" />
      <path d="M4 9.500h10a6 6 0 0 1 0 12h-3" />
    </svg>
  );
}

export function RedoIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="m16 5 4.500 4.500L16 14" />
      <path d="M20 9.500H10a6 6 0 0 0 0 12h3" />
    </svg>
  );
}

export function FolderIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="M3 7a2 2 0 0 1 2-2h4.500l2 2.500H19a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" />
    </svg>
  );
}

export function SunIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2.500v2.500M12 19v2.500M2.500 12H5M19 12h2.500M5.300 5.300 7 7M17 17l1.700 1.700M5.300 18.700 7 17M17 7l1.700-1.700" />
    </svg>
  );
}

export function MoonIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <path d="M20 14.500A8.500 8.500 0 0 1 9.500 4a8.500 8.500 0 1 0 10.500 10.500Z" />
    </svg>
  );
}

export function ChecklistIcon(props: IconProps): ReactElement {
  return (
    <svg {...base(props)}>
      <rect x="4" y="3.500" width="16" height="17" rx="2.500" />
      <path d="m8 9 1.500 1.500L12.500 7.500M8 15l1.500 1.500 3-3M15 9.500h1.500M15 15.500h1.500" />
    </svg>
  );
}

export function StarIcon({ size = 14, ...rest }: IconProps): ReactElement {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden focusable={false} {...rest}>
      <path
        fill="currentColor"
        d="m12 2.500 2.900 6.200 6.600.800-4.900 4.600 1.300 6.600L12 17.400l-5.900 3.300 1.300-6.600L2.500 9.500l6.600-.800L12 2.500Z"
      />
    </svg>
  );
}
