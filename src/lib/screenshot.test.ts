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
});
