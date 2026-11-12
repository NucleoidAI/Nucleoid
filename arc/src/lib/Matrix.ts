export function crop(matrix) {
  return locate(matrix).matrix;
}

// A drawing pairs a matrix with its epoch, the column and row of the cell at
// position (0, 0), so positions left of or above the epoch are negative. A
// matrix by itself is a drawing whose epoch is its top-left cell.
function drawing(value) {
  if (!Array.isArray(value?.matrix)) {
    return { matrix: value, epoch: { x: 0, y: 0 } };
  }
  if (!Number.isInteger(value.epoch?.x) || !Number.isInteger(value.epoch?.y)) {
    throw new Error(
      `Drawing epoch ${JSON.stringify(value.epoch)} must be integers`
    );
  }
  return value;
}

export function locate(value) {
  const { matrix, epoch } = drawing(value);
  const rows: number = matrix.length;
  const cols: number = matrix[0]?.length ?? 0;
  let top: number = rows;
  let bottom: number = -1;
  let left: number = cols;
  let right: number = -1;

  for (let y: number = 0; y < rows; y++) {
    const row = matrix[y];
    if (row.length !== cols) {
      throw new Error("Matrix must be rectangular for cropping");
    }
    for (let x: number = 0; x < cols; x++) {
      const value: number = row[x];
      if (value === 0) continue;
      top = Math.min(top, y);
      bottom = y;
      left = Math.min(left, x);
      right = Math.max(right, x);
    }
  }

  if (bottom === -1) return { matrix: [], x: 0, y: 0, width: 0, height: 0 };
  return {
    matrix: matrix
      .slice(top, bottom + 1)
      .map((row) => row.slice(left, right + 1)),
    x: left - epoch.x,
    y: top - epoch.y,
    width: right - left + 1,
    height: bottom - top + 1,
  };
}

const LETTERS: string = "ABCDEFGHI";

// Writes each color as a letter ranked by how often it occurs in the matrix:
// the most frequent color is A, the next B, and so on, while 0 stays 0. A tie
// goes to the color read first, as Map keys keep insertion order and sort is
// stable.
export function silhouette(matrix) {
  const counts = new Map<number, number>();
  for (const row of matrix) {
    for (const value of row) {
      if (!Number.isInteger(value) || value < 0 || value > 9) {
        throw new Error(
          `Matrix cell ${JSON.stringify(value)} is not a color from 0 to 9`
        );
      }
      if (value === 0) continue;
      counts.set(value, (counts.get(value) ?? 0) + 1);
    }
  }

  const ranked: number[] = [...counts.keys()].sort(
    (a: number, b: number) => counts.get(b) - counts.get(a)
  );

  return matrix.map((row) =>
    row.map((value: number) =>
      value === 0 ? 0 : LETTERS[ranked.indexOf(value)]
    )
  );
}

// Draws the nonzero cells of each placement, a matrix whose top-left cell sits
// at position (x, y), on a canvas that covers the frame and every placement,
// so a later nonzero cell wins on overlap. The epoch of the drawing it returns
// locates position (0, 0) on that canvas.
function compose(frame, placements, action: string) {
  let { left, top, right, bottom } = frame;
  for (const { matrix, x, y } of placements) {
    const cols: number = matrix[0]?.length ?? 0;
    for (const row of matrix) {
      if (row.length !== cols) {
        throw new Error(`Matrix must be rectangular for ${action}`);
      }
    }
    left = Math.min(left, x);
    top = Math.min(top, y);
    right = Math.max(right, x + cols);
    bottom = Math.max(bottom, y + matrix.length);
  }

  const result = Array.from({ length: bottom - top }, () =>
    Array(right - left).fill(0)
  );
  for (const { matrix, x, y } of placements) {
    for (let i: number = 0; i < matrix.length; i++) {
      for (let j: number = 0; j < matrix[i].length; j++) {
        if (matrix[i][j] !== 0) {
          result[y - top + i][x - left + j] = matrix[i][j];
        }
      }
    }
  }

  return { matrix: result, epoch: { x: 0 - left, y: 0 - top } };
}

// Draws the matrix with its top-left cell at position (x, y) on a canvas that
// covers rows by cols cells from the epoch and grows in any direction to keep
// the whole matrix, so x and y may be negative.
export function draw(matrix, rows: number, cols: number, x: number, y: number) {
  if (![rows, cols].every((size) => Number.isInteger(size) && size >= 0)) {
    throw new Error("Drawing dimensions must be non-negative integers");
  }
  if (!Number.isInteger(x) || !Number.isInteger(y)) {
    throw new Error(
      `Drawing position ${JSON.stringify({ x, y })} must be integers`
    );
  }
  const frame = { left: 0, top: 0, right: cols, bottom: rows };
  return compose(frame, [{ matrix, x, y }], "drawing");
}

export function pad(
  matrix,
  rows: number,
  cols: number,
  left: number,
  top: number
) {
  if (rows < 0 || cols < 0) {
    throw new Error("Padding dimensions must be non-negative");
  }
  if (left < 0 || top < 0) {
    throw new Error("Padding offsets must be non-negative");
  }
  const frame = { left: 0, top: 0, right: cols, bottom: rows };
  return compose(frame, [{ matrix, x: left, y: top }], "padding").matrix;
}

// Turns the matrix clockwise by a multiple of 90 degrees, the direction in
// which orientation counts, so the turn from one orientation to another is
// their difference. A quarter turn swaps the row and column counts.
export function rotate(matrix, degrees: number) {
  if (!Number.isInteger(degrees) || degrees % 90 !== 0) {
    throw new Error(
      `Rotation ${JSON.stringify(degrees)} is not a multiple of 90 degrees`
    );
  }
  const rows: number = matrix.length;
  const cols: number = matrix[0]?.length ?? 0;
  for (const row of matrix) {
    if (row.length !== cols) {
      throw new Error("Matrix must be rectangular for rotating");
    }
  }

  const turns: number = (((degrees / 90) % 4) + 4) % 4;
  const quarter: boolean = turns % 2 === 1;
  return Array.from({ length: quarter ? cols : rows }, (_, y: number) =>
    Array.from({ length: quarter ? rows : cols }, (_, x: number) => {
      switch (turns) {
        case 1:
          return matrix[rows - 1 - x][y];
        case 2:
          return matrix[rows - 1 - y][cols - 1 - x];
        case 3:
          return matrix[x][cols - 1 - y];
        default:
          return matrix[y][x];
      }
    })
  );
}

// Moves the nonzero cells together so that their bounding box starts at
// position (x, y), the position locate reports, which may be negative. The
// drawing covers the cells of the matrix or drawing it moves and grows in any
// direction when the moved cells need more room.
export function move(value, x: number, y: number) {
  if (!Number.isInteger(x) || !Number.isInteger(y)) {
    throw new Error(
      `Move position ${JSON.stringify({ x, y })} must be integers`
    );
  }
  const { matrix, epoch } = drawing(value);
  const frame = {
    left: 0 - epoch.x,
    top: 0 - epoch.y,
    right: (matrix[0]?.length ?? 0) - epoch.x,
    bottom: matrix.length - epoch.y,
  };
  return compose(frame, [{ matrix: crop(matrix), x, y }], "moving");
}

// Draws the matrices and drawings over one another, lined up at their epochs,
// so zero is transparent and a later nonzero cell wins on overlap.
export function merge(...values) {
  if (values.length === 0) {
    throw new Error("At least one matrix is required for merging");
  }
  const placements = values.map((value) => {
    const { matrix, epoch } = drawing(value);
    return { matrix, x: 0 - epoch.x, y: 0 - epoch.y };
  });
  const frame = {
    left: Infinity,
    top: Infinity,
    right: -Infinity,
    bottom: -Infinity,
  };
  return compose(frame, placements, "merging");
}

export function canvas(matrix, type: string) {
  switch (type) {
    case "EXACT":
      return matrix;
    case "FIT":
      return crop(matrix);
    default:
      throw new Error(`Unknown output canvas ${JSON.stringify(type)}`);
  }
}

export function difference(matrix1, matrix2): number {
  let mismatches: number = 0;
  const rows: number = Math.max(matrix1.length, matrix2.length);

  for (let i: number = 0; i < rows; i++) {
    const row1 = matrix1[i];
    const row2 = matrix2[i];

    if (!row1 || !row2) {
      // An extra empty row is still a shape mismatch.
      mismatches += Math.max(row1?.length ?? 0, row2?.length ?? 0, 1);
      continue;
    }

    const cols: number = Math.max(row1.length, row2.length);

    for (let j: number = 0; j < cols; j++) {
      if (row1[j] !== row2[j]) {
        mismatches++;
      }
    }
  }

  return mismatches;
}

export function toString(matrix) {
  matrix.forEach((row) => {
    console.debug(`[${row.join(",")}]`);
  });
}
