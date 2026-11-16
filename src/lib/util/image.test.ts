import { describe, expect, it } from 'vitest';
import { formatBytes, imageMime } from './image';

describe('imageMime', () => {
  it('maps raster image extensions, ignoring case and folders', () => {
    expect(imageMime('a.png')).toBe('image/png');
    expect(imageMime('assets/Photo.JPG')).toBe('image/jpeg');
    expect(imageMime('x/y.v2/icon.ico')).toBe('image/x-icon');
    expect(imageMime('pic.webp')).toBe('image/webp');
  });

  it('is null for SVG, other files, dotfiles and names without an extension', () => {
    expect(imageMime('logo.svg')).toBeNull();
    expect(imageMime('notes.txt')).toBeNull();
    expect(imageMime('.png')).toBeNull();
    expect(imageMime('img.dir/Makefile')).toBeNull();
  });
});

describe('formatBytes', () => {
  it('uses bytes below 1 KB and one decimal above', () => {
    expect(formatBytes(0)).toBe('0 B');
    expect(formatBytes(1023)).toBe('1023 B');
    expect(formatBytes(1024)).toBe('1.0 KB');
    expect(formatBytes(14_540)).toBe('14.2 KB');
    expect(formatBytes(3.1 * 1024 * 1024)).toBe('3.1 MB');
    expect(formatBytes(21 * 1024 ** 3)).toBe('21.0 GB');
  });
});
