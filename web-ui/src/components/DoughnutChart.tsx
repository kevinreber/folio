import { useEffect, useRef } from "react";

interface Slice {
  label: string;
  value: number;
  color: string;
}

export function DoughnutChart({
  slices,
  total,
  size = 140,
}: {
  slices: Slice[];
  total: number;
  size?: number;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || total === 0) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const cx = size / 2;
    const cy = size / 2;
    const outerR = Math.min(cx, cy) - 4;
    const innerR = outerR * 0.6;

    ctx.clearRect(0, 0, size, size);

    let startAngle = -Math.PI / 2;
    for (const slice of slices) {
      if (slice.value === 0) continue;
      const sweep = (slice.value / total) * Math.PI * 2;
      ctx.beginPath();
      ctx.arc(cx, cy, outerR, startAngle, startAngle + sweep);
      ctx.arc(cx, cy, innerR, startAngle + sweep, startAngle, true);
      ctx.closePath();
      ctx.fillStyle = slice.color;
      ctx.fill();
      startAngle += sweep;
    }
  }, [slices, total, size]);

  return (
    <div className="doughnut-wrap">
      <canvas ref={canvasRef} width={size} height={size} />
      <div className="doughnut-center-label">
        <strong>{total}</strong>
        <br />
        <span>total</span>
      </div>
    </div>
  );
}

export function DoughnutLegend({
  slices,
  total,
  onFilter,
}: {
  slices: Slice[];
  total: number;
  onFilter?: (label: string) => void;
}) {
  return (
    <div className="doughnut-legend">
      {slices
        .filter((s) => s.value > 0)
        .map((s) => {
          const pct = Math.round((s.value / total) * 100);
          return (
            <div
              key={s.label}
              className={`doughnut-legend-item ${onFilter ? "clickable" : ""}`}
              onClick={() => onFilter?.(s.label.toLowerCase().replace(/\s/g, "_"))}
            >
              <span
                className="doughnut-legend-dot"
                style={{ background: s.color }}
              />
              {s.label}{" "}
              <span className="doughnut-legend-val">
                {s.value} ({pct}%)
              </span>
            </div>
          );
        })}
    </div>
  );
}
