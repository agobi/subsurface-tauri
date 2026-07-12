// AI-generated (Claude)
// src/lib/swipePanel.ts
export function computeActiveIndex(scrollLeft: number, containerWidth: number, panelCount: number): number {
  if (containerWidth <= 0) return 0;
  const idx = Math.round(scrollLeft / containerWidth);
  return Math.min(panelCount - 1, Math.max(0, idx));
}
