export function crop(matrix) {
  return locate(matrix).matrix;
}

export function locate(matrix) {
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

  if (bottom === -1) return { matrix: [], x: 0, y: 0 };
  return {
    matrix: matrix
      .slice(top, bottom + 1)
      .map((row) => row.slice(left, right + 1)),
    x: left,
    y: top,
  };
}

export function pad(matrix, rows, cols, left, top) {
  if (!Array.isArray(matrix)) {
    throw new Error("Matrix must be an array for padding");
  }
  if (
    !Number.isSafeInteger(rows) ||
    !Number.isSafeInteger(cols) ||
    rows < 0 ||
    cols < 0
  ) {
    throw new Error("Padding dimensions must be non-negative safe integers");
  }
  if (
    !Number.isSafeInteger(left) ||
    !Number.isSafeInteger(top) ||
    left < 0 ||
    top < 0
  ) {
    throw new Error("Padding offsets must be non-negative safe integers");
  }
  const sourceRows = matrix.length;
  const sourceCols = matrix[0]?.length ?? 0;
  if (sourceRows > rows || sourceCols > cols) {
    throw new Error("Padding dimensions must not be smaller than the matrix");
  }
  if (left > cols - sourceCols || top > rows - sourceRows) {
    throw new Error("Matrix placement exceeds the padding dimensions");
  }
  const result = Array.from({ length: rows }, () => Array(cols).fill(0));

  for (let y = 0; y < sourceRows; y++) {
    const row = matrix[y];
    if (!Array.isArray(row) || row.length !== sourceCols) {
      throw new Error("Matrix must be rectangular for padding");
    }
    for (let x = 0; x < sourceCols; x++) {
      const value = row[x];
      if (!Number.isFinite(value)) {
        throw new Error("Matrix must contain finite numbers for padding");
      }
      result[y + top][x + left] = value;
    }
  }

  return result;
}

export function merge(...matrices) {
  const numRows = matrices[0].length;
  const numCols = matrices[0][0].length;

  const result = Array.from({ length: numRows }, () => Array(numCols).fill(0));

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

export function difference(matrix1, matrix2) {
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

export function toString(matrix) {
  matrix.forEach((row) => {
    console.debug(`[${row.join(",")}]`);
  });
}
