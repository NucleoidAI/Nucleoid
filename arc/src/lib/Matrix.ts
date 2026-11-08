export function crop(matrix) {
  return locate(matrix).matrix;
}

export function locate(matrix) {
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
    x: left,
    y: top,
    width: right - left + 1,
    height: bottom - top + 1,
  };
}

const LETTERS: string = "ABCDEFGHI";
const ALPHABET: string = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

type Color = number | string;

// A cell is a color from 0 to 9, or one of the letters A to Z that the
// instruction datasets write their nonzero colors as, each dataset with its
// own random letters.
function color(value: unknown): value is Color {
  return (
    (typeof value === "number" &&
      Number.isInteger(value) &&
      value >= 0 &&
      value <= 9) ||
    (typeof value === "string" &&
      value.length === 1 &&
      ALPHABET.includes(value))
  );
}

// Writes each color as a letter ranked by how often it occurs in the matrix:
// the most frequent color is A, the next B, and so on, while 0 stays 0. A tie
// goes to the color read first, as Map keys keep insertion order and sort is
// stable. Cells that are already letters, as in the instruction datasets, are
// ranked the same way, so a crop is ranked by its own counts.
export function silhouette(matrix) {
  const counts = new Map<Color, number>();
  for (const row of matrix) {
    for (const value of row) {
      if (!color(value)) {
        throw new Error(
          `Matrix cell ${JSON.stringify(
            value
          )} is not a color from 0 to 9 or a letter from A to Z`
        );
      }
      if (value === 0) continue;
      counts.set(value, (counts.get(value) ?? 0) + 1);
    }
  }

  const ranked: Color[] = [...counts.keys()].sort(
    (a: Color, b: Color) => counts.get(b) - counts.get(a)
  );
  if (ranked.length > LETTERS.length) {
    throw new Error(
      `Matrix holds ${ranked.length} colors, more than ${LETTERS.length}`
    );
  }

  return matrix.map((row) =>
    row.map((value: Color) =>
      value === 0 ? 0 : LETTERS[ranked.indexOf(value)]
    )
  );
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
  const sourceRows: number = matrix.length;
  const sourceCols: number = matrix[0]?.length ?? 0;
  const paddedRows: number = Math.max(rows, top + sourceRows);
  const paddedCols: number = Math.max(cols, left + sourceCols);
  const result = Array.from({ length: paddedRows }, () =>
    Array(paddedCols).fill(0)
  );

  for (let y: number = 0; y < sourceRows; y++) {
    const row = matrix[y];
    if (row.length !== sourceCols) {
      throw new Error("Matrix must be rectangular for padding");
    }
    for (let x: number = 0; x < sourceCols; x++) {
      result[y + top][x + left] = row[x];
    }
  }

  return result;
}

export function merge(...matrices) {
  if (matrices.length === 0) {
    throw new Error("At least one matrix is required for merging");
  }
  const numRows: number = Math.max(...matrices.map((matrix) => matrix.length));
  const numCols: number = Math.max(
    ...matrices.map((matrix) => matrix[0]?.length ?? 0)
  );

  const result = Array.from({ length: numRows }, () => Array(numCols).fill(0));

  for (const matrix of matrices) {
    const padded = pad(matrix, numRows, numCols, 0, 0);
    for (let i: number = 0; i < numRows; i++) {
      for (let j: number = 0; j < numCols; j++) {
        if (padded[i][j] !== 0) {
          result[i][j] = padded[i][j];
        }
      }
    }
  }

  return result;
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
