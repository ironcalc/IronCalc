// Positions of rows or columns along one axis of a sheet.
//
// A sheet has 1,048,576 rows, almost all of the default height, so sizes are
// stored as runs of consecutive indices with the same non-default size (hidden
// ones have size 0) with the pixel offset where each run starts. Index to
// offset and offset to index are binary searches.
//
// A port of the desktop app's `axis.rs`.

import type { AxisSizes } from "@ironcalc/wasm";

interface Run {
  // First and last index (1-based, inclusive) with this size
  first: number;
  last: number;
  size: number;
  // Offset of `first`
  start: number;
}

function runEnd(run: Run): number {
  return run.start + (run.last - run.first + 1) * run.size;
}

// The number of leading elements for which `predicate` holds (the elements
// must be partitioned: all those satisfying it come first).
function partitionPoint<T>(
  items: T[],
  predicate: (item: T) => boolean,
): number {
  let low = 0;
  let high = items.length;
  while (low < high) {
    const mid = (low + high) >>> 1;
    if (predicate(items[mid])) {
      low = mid + 1;
    } else {
      high = mid;
    }
  }
  return low;
}

export class Axis {
  private readonly defaultSize: number;
  // 1-based index of the last row or column
  readonly count: number;
  // Sorted by `first`, non-overlapping, all with `size !== defaultSize`
  private readonly runs: Run[];

  // `sizes` are [first, last, size] ranges in any order; ranges equal to the
  // default are dropped and adjacent equal ones merged. Overlapping ranges
  // are not expected (the engine keeps rows unique and column ranges
  // disjoint); the earlier one wins where they do.
  constructor(
    defaultSize: number,
    count: number,
    sizes: Iterable<[number, number, number]>,
  ) {
    const ranges: [number, number, number][] = [];
    for (const [first, last, size] of sizes) {
      if (first > last || last < 1 || first > count || size === defaultSize) {
        continue;
      }
      ranges.push([
        Math.max(first, 1),
        Math.min(last, count),
        Math.max(size, 0),
      ]);
    }
    ranges.sort((a, b) => a[0] - b[0]);
    const runs: Run[] = [];
    for (const [rangeFirst, last, size] of ranges) {
      const previous = runs.length > 0 ? runs[runs.length - 1] : null;
      // Trim anything the previous run already covers
      const first = previous
        ? Math.max(rangeFirst, previous.last + 1)
        : rangeFirst;
      if (first > last) {
        continue;
      }
      if (previous && previous.last + 1 === first && previous.size === size) {
        previous.last = last;
        continue;
      }
      const start = previous
        ? runEnd(previous) + (first - previous.last - 1) * defaultSize
        : (first - 1) * defaultSize;
      runs.push({ first, last, size, start });
    }
    this.defaultSize = defaultSize;
    this.count = count;
    this.runs = runs;
  }

  static uniform(defaultSize: number, count: number): Axis {
    return new Axis(defaultSize, count, []);
  }

  // Builds the axis from the engine's sizes (see `Model.getRowSizes` and
  // `Model.getColumnSizes`). Sizes are rounded to whole pixels, as the
  // canvas rounds every row height and column width it draws.
  static fromSizes(sizes: AxisSizes, count: number): Axis {
    return new Axis(
      Math.round(sizes.default_size),
      count,
      sizes.runs.map(({ first, last, size }) => [
        first,
        last,
        Math.round(size),
      ]),
    );
  }

  // The run containing `index`, or the last run before it
  private runBefore(index: number): Run | null {
    const i = partitionPoint(this.runs, (run) => run.first <= index);
    return i > 0 ? this.runs[i - 1] : null;
  }

  // Offset of the leading edge of `index` (1-based). `index === count + 1`
  // gives the total extent.
  start(index: number): number {
    const clamped = Math.min(Math.max(index, 1), this.count + 1);
    const run = this.runBefore(clamped);
    if (!run) {
      return (clamped - 1) * this.defaultSize;
    }
    if (clamped <= run.last) {
      return run.start + (clamped - run.first) * run.size;
    }
    return runEnd(run) + (clamped - run.last - 1) * this.defaultSize;
  }

  size(index: number): number {
    const run = this.runBefore(index);
    if (run && index <= run.last) {
      return run.size;
    }
    return this.defaultSize;
  }

  total(): number {
    return this.start(this.count + 1);
  }

  // The index whose span contains `offset`, clamped to the axis. Hidden
  // (zero-size) indices share their spot with the next visible one, which is
  // the one returned.
  at(offset: number): number {
    if (offset <= 0) {
      return 1;
    }
    const i = partitionPoint(this.runs, (run) => run.start <= offset);
    const run = i > 0 ? this.runs[i - 1] : null;
    let index: number;
    if (!run) {
      index = 1 + Math.floor(offset / this.defaultSize);
    } else if (offset < runEnd(run)) {
      index =
        run.size > 0
          ? run.first + Math.floor((offset - run.start) / run.size)
          : run.first;
    } else {
      index =
        run.last + 1 + Math.floor((offset - runEnd(run)) / this.defaultSize);
    }
    return Math.min(Math.max(index, 1), this.count);
  }

  // The nearest index at or after `index` with a non-zero size, if any
  nextVisible(index: number): number | null {
    let i = Math.max(index, 1);
    while (i <= this.count) {
      if (this.size(i) > 0) {
        return i;
      }
      const run = this.runBefore(i);
      i = run && i <= run.last ? run.last + 1 : i + 1;
    }
    return null;
  }

  // The nearest index at or before `index` with a non-zero size, if any
  prevVisible(index: number): number | null {
    let i = Math.min(index, this.count);
    while (i >= 1) {
      if (this.size(i) > 0) {
        return i;
      }
      const run = this.runBefore(i);
      i = run && i <= run.last ? run.first - 1 : i - 1;
    }
    return null;
  }
}
