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

/// Converts a CSS-pixel selection into the original screenshot's pixel space.
/// The overlay displays the image stretched to the window, so the ratio between
/// natural size and rendered size is the display scale factor.
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
