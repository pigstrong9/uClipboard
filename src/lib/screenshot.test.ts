import { describe, expect, it } from 'vitest';

import { selectionToPixels } from './screenshot';

describe('selectionToPixels', () => {
  it('maps css pixels to image pixels at scale 1', () => {
    expect(
      selectionToPixels({ left: 10, top: 20, width: 100, height: 50 }, 1920, 1080, 1920, 1080),
    ).toEqual({ x: 10, y: 20, width: 100, height: 50 });
  });

  it('accounts for display scaling', () => {
    expect(
      selectionToPixels({ left: 100, top: 50, width: 200, height: 100 }, 2000, 1200, 1000, 600),
    ).toEqual({ x: 200, y: 100, width: 400, height: 200 });
  });

  it('clamps selections to the image bounds', () => {
    expect(
      selectionToPixels({ left: -20, top: 10, width: 50, height: 2000 }, 100, 100, 100, 100),
    ).toEqual({ x: 0, y: 10, width: 30, height: 90 });
    expect(
      selectionToPixels({ left: 200, top: 200, width: 10, height: 10 }, 100, 100, 100, 100),
    ).toBeNull();
  });

  it('maps a Retina overlay through its own pixel density', () => {
    // 1512x982 points rendered from a 3024x1964 pixel capture.
    expect(
      selectionToPixels({ left: 100, top: 50, width: 200, height: 100 }, 3024, 1964, 1512, 982),
    ).toEqual({ x: 200, y: 100, width: 400, height: 200 });
  });

  it('keeps each display independent when densities differ', () => {
    const retina = { image: [3024, 1964] as const, view: [1512, 982] as const };
    const external = { image: [1920, 1080] as const, view: [1920, 1080] as const };
    const box = { left: 0, top: 0, width: 200, height: 100 };
    // The same CSS rectangle covers different pixel regions on the two displays, and
    // neither mapping can reach into the other display's capture.
    expect(selectionToPixels(box, ...retina.image, ...retina.view)).toEqual({
      x: 0,
      y: 0,
      width: 400,
      height: 200,
    });
    expect(selectionToPixels(box, ...external.image, ...external.view)).toEqual({
      x: 0,
      y: 0,
      width: 200,
      height: 100,
    });
  });

  it('rejects a view or image with no size', () => {
    expect(selectionToPixels({ left: 0, top: 0, width: 10, height: 10 }, 100, 100, 0, 100)).toBeNull();
    expect(selectionToPixels({ left: 0, top: 0, width: 10, height: 10 }, 0, 100, 100, 100)).toBeNull();
  });
});
