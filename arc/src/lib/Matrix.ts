export function crop(matrix) {
  return measure(matrix).matrix;
}

export function measure(matrix) {
  const rows: number = matrix.length;
  const cols: number = matrix[0]?.length ?? 0;
  let top: number = rows;
  let bottom: number = -1;
  let left: number = cols;
  let right: number = -1;
  let volume: number = 0;

  for (let y: number = 0; y < rows; y++) {
    const row = matrix[y];
    if (row.length !== cols) {
      throw new Error("Matrix must be rectangular for cropping");
    }
    for (let x: number = 0; x < cols; x++) {
      const value: number = row[x];
      if (value === 0) continue;
      volume++;
      top = Math.min(top, y);
      bottom = y;
      left = Math.min(left, x);
      right = Math.max(right, x);
    }
  }

  if (bottom === -1) {
    return { matrix: [], x: 0, y: 0, width: 0, height: 0, volume: 0 };
  }
  return {
    matrix: matrix
      .slice(top, bottom + 1)
      .map((row) => row.slice(left, right + 1)),
    x: left,
    y: top,
    width: right - left + 1,
    height: bottom - top + 1,
    volume,
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

// Turns the instance inside the matrix, all of its nonzero cells, clockwise by
// a multiple of 90 degrees, the direction in which orientation counts, so the
// turn from one orientation to another is their difference. The matrix is
// cropped to the instance, which turns in place: its bounding box keeps the
// top-left corner that measure reports, and a quarter turn swaps its width and
// height. The canvas keeps the matrix's size and grows right or down, as pad
// does, when the turned instance needs more room.
export function rotate_instance(matrix, degrees: number) {
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

  const { matrix: instance, x, y, width, height } = measure(matrix);
  const turns: number = (((degrees / 90) % 4) + 4) % 4;
  const quarter: boolean = turns % 2 === 1;
  const rotated = Array.from(
    { length: quarter ? width : height },
    (_, row: number) =>
      Array.from({ length: quarter ? height : width }, (_, column: number) => {
        switch (turns) {
          case 1:
            return instance[height - 1 - column][row];
          case 2:
            return instance[height - 1 - row][width - 1 - column];
          case 3:
            return instance[column][width - 1 - row];
          default:
            return instance[row][column];
        }
      })
  );
  return pad(rotated, rows, cols, x, y);
}

// Pushes the instance inside the matrix, all of its nonzero cells, by x columns
// and y rows. The matrix is cropped to the instance, whose bounding box leaves
// the position measure reports by that offset; negative offsets push left or
// up. The canvas keeps the matrix's size and grows right or down, as pad does,
// when the pushed instance needs more room.
export function move_instance(matrix, x: number, y: number) {
  const offset: string = JSON.stringify({ x, y });
  if (!Number.isInteger(x) || !Number.isInteger(y)) {
    throw new Error(`Move offset ${offset} must be integers`);
  }
  const rows: number = matrix.length;
  const cols: number = matrix[0]?.length ?? 0;
  const { matrix: instance, x: left, y: top, volume } = measure(matrix);
  if (volume === 0) {
    return pad(instance, rows, cols, 0, 0);
  }
  if (left + x < 0 || top + y < 0) {
    throw new Error(
      `Move offset ${offset} pushes the instance past the left or top edge`
    );
  }
  return pad(instance, rows, cols, left + x, top + y);
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

// Lists the steps from a cell to every cell at most reach steps away, moving
// along sides only with connectivity 4 and across corners as well with 8.
function neighborhood(connectivity: number, reach: number): number[][] {
  const steps: number[][] = [];
  for (let y: number = -reach; y <= reach; y++) {
    for (let x: number = -reach; x <= reach; x++) {
      const distance: number =
        connectivity === 8
          ? Math.max(Math.abs(x), Math.abs(y))
          : Math.abs(x) + Math.abs(y);
      if (distance > 0 && distance <= reach) steps.push([x, y]);
    }
  }
  return steps;
}

// Separates the nonzero cells of the matrix into groups of adjacent cells, one
// matrix per group. Connectivity 4 joins cells that share a side, and 8 also
// joins cells that share only a corner. Any two adjacent nonzero cells join,
// whatever their colors, unless same_color asks them to have the same color.
// A gap lets cells join across up to that many cells between them, whatever
// those cells hold: two cells join when one is at most gap + 1 steps from the
// other, so a gap of 0 joins only adjacent cells. Each group keeps the
// matrix's size, with its cells where they are and 0 everywhere else, as
// instance matrices do, so merge puts the groups back together. Groups come in
// reading order of their first cell.
export function separate(
  matrix,
  connectivity: number = 8,
  same_color: boolean = false,
  gap: number = 0
) {
  if (connectivity !== 4 && connectivity !== 8) {
    throw new Error(
      `Connectivity ${JSON.stringify(connectivity)} is not 4 or 8`
    );
  }
  if (!Number.isInteger(gap) || gap < 0) {
    throw new Error(
      `Gap ${JSON.stringify(gap)} must be an integer of 0 or more`
    );
  }
  const rows: number = matrix.length;
  const cols: number = matrix[0]?.length ?? 0;
  for (const row of matrix) {
    if (row.length !== cols) {
      throw new Error("Matrix must be rectangular for separating");
    }
  }

  const neighbors: number[][] = neighborhood(connectivity, gap + 1);
  const grouped = Array.from({ length: rows }, () => Array(cols).fill(false));
  const groups = [];

  for (let y: number = 0; y < rows; y++) {
    for (let x: number = 0; x < cols; x++) {
      if (matrix[y][x] === 0 || grouped[y][x]) continue;

      const group = Array.from({ length: rows }, () => Array(cols).fill(0));
      const pending: number[][] = [[x, y]];
      grouped[y][x] = true;

      while (pending.length > 0) {
        const [cell_x, cell_y] = pending.pop();
        group[cell_y][cell_x] = matrix[cell_y][cell_x];

        for (const [step_x, step_y] of neighbors) {
          const next_x: number = cell_x + step_x;
          const next_y: number = cell_y + step_y;
          if (next_x < 0 || next_y < 0 || next_x >= cols || next_y >= rows) {
            continue;
          }
          const value: number = matrix[next_y][next_x];
          if (value === 0 || grouped[next_y][next_x]) continue;
          if (same_color && value !== matrix[cell_y][cell_x]) continue;
          grouped[next_y][next_x] = true;
          pending.push([next_x, next_y]);
        }
      }

      groups.push(group);
    }
  }

  return groups;
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
