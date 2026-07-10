// AI-generated (Claude)
// src/lib/swipePanel.ts

// Only commit to a neighboring panel once the scroll has crossed `thresholdFraction`
// of the container width away from the panel the gesture started on; otherwise snap
// back to the starting panel. This is what makes the swipe feel deliberate rather than
// jumping to the next panel on a light touch.
export function computeSnapTarget(
  scrollLeft: number,
  startIndex: number,
  containerWidth: number,
  panelCount: number,
  thresholdFraction: number,
): number {
  if (containerWidth <= 0) return Math.min(panelCount - 1, Math.max(0, startIndex));
  const fraction = (scrollLeft - startIndex * containerWidth) / containerWidth;
  if (fraction > thresholdFraction) return Math.min(panelCount - 1, startIndex + 1);
  if (fraction < -thresholdFraction) return Math.max(0, startIndex - 1);
  return startIndex;
}
