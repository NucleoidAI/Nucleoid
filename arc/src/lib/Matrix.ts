import type { Grid } from "../types";

let _rows = 0;
let _cols = 0;

export function init(length: number): void {
  _rows = length;
  _cols = length;
}

export function crop(matrix: Grid): Grid {
  if (!Array.isArray(matrix)) {
    throw new Error("Matrix must be an array for cropping");
  }
  const rows = matrix.length;
  const cols = matrix[0]?.length ?? 0;
  let top = rows;
  let bottom = -1;
  let left = cols;
  let right = -1;

  for (let y = 0; y < rows; y++) {
    const row = matrix[y];
    if (!Array.isArray(row) || row.length !== cols) {
      throw new Error("Matrix must be rectangular for cropping");
    }
    for (let x = 0; x < cols; x++) {
      const value = row[x];
      if (!Number.isFinite(value)) {
        throw new Error("Matrix must contain finite numbers for cropping");
      }
      if (value === 0) continue;
      top = Math.min(top, y);
      bottom = y;
      left = Math.min(left, x);
      right = Math.max(right, x);
    }
  }

  if (bottom === -1) return [];
  return matrix.slice(top, bottom + 1).map((row) => row.slice(left, right + 1));
}

export function merge(...matrices: Grid[]): Grid {
  const numRows = matrices[0].length;
  const numCols = matrices[0][0].length;

  const result = Array.from({ length: numRows }, () =>
    Array<number>(numCols).fill(0)
  );

  for (const matrix of matrices) {
    for (let i = 0; i < numRows; i++) {
      for (let j = 0; j < numCols; j++) {
        if (matrix[i][j] !== 0) {
          result[i][j] = matrix[i][j];
        }
      }
    }
  }

  return result;
}

export function difference(matrix1: Grid, matrix2: Grid): number {
  let mismatches = 0;
  const rows = Math.max(matrix1.length, matrix2.length);

  for (let i = 0; i < rows; i++) {
    const row1 = matrix1[i];
    const row2 = matrix2[i];

    if (!row1 || !row2) {
      // An extra empty row is still a shape mismatch.
      mismatches += Math.max(row1?.length ?? 0, row2?.length ?? 0, 1);
      continue;
    }

    const cols = Math.max(row1.length, row2.length);

    for (let j = 0; j < cols; j++) {
      if (row1[j] !== row2[j]) {
        mismatches++;
      }
    }
  }

  return mismatches;
}

export function subtract(matrix1: Grid, matrix2: Grid): Grid {
  const rows = matrix1.length;
  const cols = matrix1[0].length;

  // Ensure both matrices have the same dimensions
  if (matrix2.length !== rows || matrix2[0].length !== cols) {
    throw new Error("Matrices must have the same dimensions for subtraction");
  }

  // Initialize a result matrix
  const result = Array.from({ length: rows }, () =>
    Array<number>(cols).fill(0)
  );

  // Perform element-wise subtraction
  for (let i = 0; i < rows; i++) {
    for (let j = 0; j < cols; j++) {
      const num = matrix1[i][j] - matrix2[i][j];
      result[i][j] = num > 0 ? num : 0;
    }
  }

  return result;
}

export function encode(matrix: Grid): Grid {
  return matrix;
}

export function decode(matrix: Grid): Grid {
  return matrix;
}

// Experimental encoding/decoding
/* eslint-disable @typescript-eslint/no-unused-vars */

function sparse_encode(matrix: Grid): string {
  const result: string[] = [];

  for (let i = 0; i < matrix.length; i++) {
    for (let j = 0; j < matrix[i].length; j++) {
      if (matrix[i][j] !== 0) {
        result.push(`${matrix[i][j]}@${i},${j}`);
      }
    }
  }

  return result.join("-");
}

function sparse_decode(string: string): Grid {
  const matrix = Array.from({ length: _rows }, () =>
    Array<number>(_cols).fill(0)
  );

  const entries = string.split("-");

  entries.forEach((entry) => {
    const [value, position] = entry.split("@");
    const [row, col] = position.split(",").map(Number);
    matrix[row][col] = Number(value);
  });

  return matrix;
}

export function toString(matrix: Grid): void {
  decode(matrix).forEach((row) => {
    console.debug(`[${row.join(",")}]`);
  });
}
