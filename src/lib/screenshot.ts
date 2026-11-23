export interface SelectionBox {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface PixelRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * Converts a CSS-pixel selection into the pixels of the capture shown by one overlay.
 *
 * Each display keeps its own capture at its own pixel resolution, so the ratio between
 * the image size and the rendered size is that display's scale factor. Sizing the window
 * differently — or a resize that has not landed yet — only changes `displayWidth` and
 * `displayHeight`, never the mapping.
 */
export function selectionToPixels(
  selection: SelectionBox,
  imageWidth: number,
  imageHeight: number,
  displayWidth: number,
  displayHeight: number,
): PixelRect | null {
  if (imageWidth <= 0 || imageHeight <= 0 || displayWidth <= 0 || displayHeight <= 0) {
    return null;
  }
  const scaleX = imageWidth / displayWidth;
  const scaleY = imageHeight / displayHeight;
  const left = Math.min(selection.left, selection.left + selection.width);
  const top = Math.min(selection.top, selection.top + selection.height);
  const right = Math.max(selection.left, selection.left + selection.width);
  const bottom = Math.max(selection.top, selection.top + selection.height);

  const x0 = Math.max(0, Math.round(left * scaleX));
  const y0 = Math.max(0, Math.round(top * scaleY));
  const x1 = Math.min(imageWidth, Math.round(right * scaleX));
  const y1 = Math.min(imageHeight, Math.round(bottom * scaleY));
  if (x1 <= x0 || y1 <= y0) return null;
  return { x: x0, y: y0, width: x1 - x0, height: y1 - y0 };
}
