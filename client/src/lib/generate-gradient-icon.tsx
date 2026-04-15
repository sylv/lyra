type GenerateGradientIconOptions = {
  size?: number;
};

const DEFAULT_SIZE = 256;
const MIN_SIZE = 16;
const MAX_SIZE = 1024;

const iconCache = new Map<string, string>();

const clamp = (value: number, min: number, max: number): number => Math.min(Math.max(value, min), max);

const wrapHue = (hue: number): number => {
  const value = hue % 360;
  return value < 0 ? value + 360 : value;
};

const hashSeed = (seed: string): number => {
  let hash = 2166136261;
  for (let i = 0; i < seed.length; i += 1) {
    hash ^= seed.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
};

const createRandom = (seed: number): (() => number) => {
  let value = seed || 0x6d2b79f5;
  return () => {
    value += 0x6d2b79f5;
    let t = value;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
};

const randomInRange = (random: () => number, min: number, max: number): number => min + random() * (max - min);

const colorFromHsl = (hue: number, saturation: number, lightness: number, alpha = 1): string =>
  `hsla(${Math.round(wrapHue(hue))}, ${Math.round(saturation)}%, ${Math.round(lightness)}%, ${alpha})`;

const getSvgDataUrl = (svg: string): string => `data:image/svg+xml,${encodeURIComponent(svg)}`;

export const generateGradientIcon = (seed: string, options: GenerateGradientIconOptions = {}): string => {
  const safeSeed = seed.length > 0 ? seed : "lyra";
  const size = Math.round(clamp(options.size ?? DEFAULT_SIZE, MIN_SIZE, MAX_SIZE));
  const cacheKey = `${safeSeed}:${size}`;

  const cached = iconCache.get(cacheKey);
  if (cached) {
    return cached;
  }

  const random = createRandom(hashSeed(safeSeed));
  const baseHue = randomInRange(random, 0, 360);
  const hueB = baseHue + randomInRange(random, 36, 96);
  const hueC = baseHue + randomInRange(random, 150, 235);
  const hueD = baseHue + randomInRange(random, 260, 320);

  const colorA = colorFromHsl(baseHue, randomInRange(random, 72, 94), randomInRange(random, 50, 62));
  const colorB = colorFromHsl(hueB, randomInRange(random, 70, 92), randomInRange(random, 54, 66));
  const colorC = colorFromHsl(hueC, randomInRange(random, 66, 90), randomInRange(random, 46, 60));
  const colorD = colorFromHsl(hueD, randomInRange(random, 68, 92), randomInRange(random, 58, 70));

  const bgAngle = randomInRange(random, 0, Math.PI * 2);
  const x1 = 50 + Math.cos(bgAngle) * 50;
  const y1 = 50 + Math.sin(bgAngle) * 50;
  const x2 = 50 - Math.cos(bgAngle) * 50;
  const y2 = 50 - Math.sin(bgAngle) * 50;

  const blur = Math.max(8, Math.round(size * 0.16));
  const blobs = Array.from({ length: 4 }, (_, index) => {
    const cx = randomInRange(random, size * 0.15, size * 0.85);
    const cy = randomInRange(random, size * 0.15, size * 0.85);
    const rx = randomInRange(random, size * 0.22, size * 0.42);
    const ry = randomInRange(random, size * 0.2, size * 0.4);
    const rotation = randomInRange(random, 0, 360);
    const opacity = randomInRange(random, 0.65, 0.9);
    const fill = [colorB, colorC, colorD, colorA][index];
    return `<ellipse cx="${cx.toFixed(2)}" cy="${cy.toFixed(2)}" rx="${rx.toFixed(2)}" ry="${ry.toFixed(2)}" fill="${fill}" fill-opacity="${opacity.toFixed(2)}" transform="rotate(${rotation.toFixed(2)} ${cx.toFixed(2)} ${cy.toFixed(2)})" />`;
  }).join("");

  const highlightCx = randomInRange(random, 18, 34);
  const highlightCy = randomInRange(random, 14, 30);
  const vignetteOpacity = randomInRange(random, 0.12, 0.22);

  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" fill="none" preserveAspectRatio="none"><defs><linearGradient id="bg" x1="${x1.toFixed(2)}%" y1="${y1.toFixed(2)}%" x2="${x2.toFixed(2)}%" y2="${y2.toFixed(2)}%"><stop offset="0%" stop-color="${colorA}" /><stop offset="54%" stop-color="${colorB}" /><stop offset="100%" stop-color="${colorC}" /></linearGradient><radialGradient id="highlight" cx="${highlightCx.toFixed(2)}%" cy="${highlightCy.toFixed(2)}%" r="72%"><stop offset="0%" stop-color="#fff" stop-opacity="0.38" /><stop offset="100%" stop-color="#fff" stop-opacity="0" /></radialGradient><radialGradient id="vignette" cx="50%" cy="50%" r="74%"><stop offset="40%" stop-color="#000" stop-opacity="0" /><stop offset="100%" stop-color="#000" stop-opacity="${vignetteOpacity.toFixed(2)}" /></radialGradient><filter id="blur"><feGaussianBlur stdDeviation="${blur}" /></filter></defs><rect width="${size}" height="${size}" fill="url(#bg)" /><g filter="url(#blur)">${blobs}</g><rect width="${size}" height="${size}" fill="url(#highlight)" /><rect width="${size}" height="${size}" fill="url(#vignette)" /></svg>`;

  const url = getSvgDataUrl(svg);
  iconCache.set(cacheKey, url);
  return url;
};
