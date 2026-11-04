import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';

const tokens = readFileSync('src/styles/tokens.css', 'utf8');

const palette = new Map(
  [...tokens.split('@media')[0].matchAll(/(--[\w-]+):\s*([^;]+);/g)].map(([, name, value]) => [
    name,
    value.trim(),
  ]),
);

function color(name: string, background = [0, 0, 0]): number[] {
  const value = palette.get(name);
  if (!value) throw new Error(`Missing token ${name}`);
  if (value.startsWith('#')) {
    return [1, 3, 5].map((offset) => parseInt(value.slice(offset, offset + 2), 16));
  }
  const channels = value.match(/[\d.]+/g)?.map(Number);
  if (!channels || channels.length !== 4) throw new Error(`Unsupported color ${value}`);
  return channels
    .slice(0, 3)
    .map((channel, index) => channel * channels[3] + background[index] * (1 - channels[3]));
}

function luminance(rgb: number[]): number {
  return rgb
    .map((channel) => {
      const srgb = channel / 255;
      return srgb <= 0.04045 ? srgb / 12.92 : ((srgb + 0.055) / 1.055) ** 2.4;
    })
    .reduce((sum, channel, index) => sum + channel * [0.2126, 0.7152, 0.0722][index], 0);
}

function contrast(foreground: number[], background: number[]): number {
  const values = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (values[0] + 0.05) / (values[1] + 0.05);
}

describe('Reid small-text contrast', () => {
  const surfaces = [
    '--color-content',
    '--color-surface',
    '--color-surface-raised',
    '--color-sidebar',
    '--color-inspector',
    '--color-sidebar-glass',
    '--color-inspector-glass',
  ];

  for (const surface of surfaces) {
    for (const text of [
      '--text-primary',
      '--text-secondary',
      '--text-metadata',
      '--text-success',
      '--color-danger',
      '--accent-action',
    ]) {
      it(`${text} remains readable on ${surface}`, () => {
        const background = color(surface, color('--color-content'));
        expect(contrast(color(text, background), background)).toBeGreaterThanOrEqual(4.5);
      });
    }
  }

  for (const fill of ['--accent-action', '--accent-hover']) {
    it(`primary button text remains readable on ${fill}`, () => {
      expect(contrast(color('--text-on-accent'), color(fill))).toBeGreaterThanOrEqual(4.5);
    });
  }
});
