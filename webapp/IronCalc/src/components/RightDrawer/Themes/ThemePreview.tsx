import type { ThemeData } from "./EditTheme";

function polarToCartesian(
  cx: number,
  cy: number,
  r: number,
  angleDeg: number,
): [number, number] {
  const rad = ((angleDeg - 90) * Math.PI) / 180;
  return [cx + r * Math.cos(rad), cy + r * Math.sin(rad)];
}

function sectorPath(
  cx: number,
  cy: number,
  r: number,
  startDeg: number,
  endDeg: number,
): string {
  const [sx, sy] = polarToCartesian(cx, cy, r, startDeg);
  const [ex, ey] = polarToCartesian(cx, cy, r, endDeg);
  const large = endDeg - startDeg > 180 ? 1 : 0;
  return `M${cx},${cy} L${sx},${sy} A${r},${r},0,${large},1,${ex},${ey}Z`;
}

interface ThemePreviewProps {
  theme: Pick<ThemeData, "bgColor" | "textColor" | "accentColors">;
  className: string;
}

const ThemePreview = ({ theme, className }: ThemePreviewProps) => (
  <div
    className={className}
    style={
      {
        backgroundColor: theme.bgColor,
        color: theme.textColor,
        "--theme-bg": theme.bgColor,
      } as React.CSSProperties
    }
  >
    <div className="ic-theme-preview-inner">
      <span>Aa</span>
      <svg
        viewBox="0 0 28 28"
        className="ic-theme-preview-pie"
        aria-hidden="true"
      >
        {theme.accentColors.map((color, i) => (
          <path
            // biome-ignore lint/suspicious/noArrayIndexKey: fixed-length array, order never changes
            key={i}
            d={sectorPath(14, 14, 13, i * 60, (i + 1) * 60)}
            fill={color}
          />
        ))}
      </svg>
    </div>
  </div>
);

export default ThemePreview;
