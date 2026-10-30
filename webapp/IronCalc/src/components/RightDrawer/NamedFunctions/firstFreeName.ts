/** The first name of the form `base`, `base_1`, `base_2`... not in `taken` */
export function firstFreeName(base: string, taken: string[]): string {
  const upper = taken.map((name) => name.toUpperCase());
  if (!upper.includes(base.toUpperCase())) {
    return base;
  }
  let counter = 1;
  while (upper.includes(`${base}_${counter}`.toUpperCase())) {
    counter += 1;
  }
  return `${base}_${counter}`;
}
