let _rows = 0;
let _cols = 0;

function init(length) {
  _rows = length;
  _cols = length;
}

function merge(...matrices) {
  const numRows = matrices[0].length;
  const numCols = matrices[0][0].length;

  let result = Array.from({ length: numRows }, () => Array(numCols).fill(0));

  for (let matrix of matrices) {
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

function subtract(matrix1, matrix2) {
  const rows = matrix1.length;
  const cols = matrix1[0].length;

  // Ensure both matrices have the same dimensions
  if (matrix2.length !== rows || matrix2[0].length !== cols) {
    throw new Error("Matrices must have the same dimensions for subtraction");
  }

  // Initialize a result matrix
  let result = Array.from({ length: rows }, () => Array(cols).fill(0));

  // Perform element-wise subtraction
  for (let i = 0; i < rows; i++) {
    for (let j = 0; j < cols; j++) {
      const num = matrix1[i][j] - matrix2[i][j];
      result[i][j] = num > 0 ? num : 0;
    }
  }

  return result;
}

function encode(matrix) {
  return matrix;
}

function decode(string) {
  return string;
}

// Experimental encoding/decoding
/* eslint-disable no-unused-vars */

function sparse_encode(matrix) {
  let result = [];

  for (let i = 0; i < matrix.length; i++) {
    for (let j = 0; j < matrix[i].length; j++) {
      if (matrix[i][j] !== 0) {
        result.push(`${matrix[i][j]}@${i},${j}`);
      }
    }
  }

  return result.join("-");
}

function sparse_decode(string) {
  let matrix = Array.from({ length: _rows }, () => Array(_cols).fill(0));

  const entries = string.split("-");

  entries.forEach((entry) => {
    const [value, position] = entry.split("@");
    const [row, col] = position.split(",").map(Number);
    matrix[row][col] = Number(value);
  });

  return matrix;
}

function toString(matrix) {
  decode(matrix).forEach((row) => {
    console.debug(`[${row.join(",")}]`);
  });
}

module.exports = { init, merge, encode, decode, subtract, toString };
