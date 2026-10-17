import assert from "node:assert/strict";
import { test } from "node:test";
import { difference } from "./Matrix";

test("difference is zero for identical matrices", () => {
  assert.equal(
    difference(
      [
        [0, 1],
        [8, 0],
      ],
      [
        [0, 1],
        [8, 0],
      ]
    ),
    0
  );
  assert.equal(difference([], []), 0);
  assert.equal(difference([[]], [[]]), 0);
});

test("difference counts wrong cells rather than numeric color distances", () => {
  assert.equal(difference([[1]], [[9]]), 1);
  assert.equal(difference([[0, 8]], [[8, 0]]), 2);
  assert.equal(
    difference(
      [
        [1, 2],
        [3, 4],
      ],
      [
        [1, 5],
        [0, 4],
      ]
    ),
    2
  );
});

test("difference counts missing and extra cells, including zero cells", () => {
  assert.equal(difference([[0, 0]], [[0]]), 1);
  assert.equal(difference([[0]], [[0, 0]]), 1);
  assert.equal(difference([[0], [0]], [[0]]), 1);
  assert.equal(difference([[0]], [[0], [0]]), 1);
  assert.equal(difference([[0, 0]], [[0], [0]]), 2);
  assert.equal(
    difference(
      [],
      [
        [0, 0],
        [0, 0],
      ]
    ),
    4
  );
});

test("difference detects empty-row and ragged shape mismatches", () => {
  assert.equal(difference([[]], []), 1);
  assert.equal(difference([], [[]]), 1);
  assert.equal(difference([[], []], [[]]), 1);
  assert.equal(difference([[1], [2, 0]], [[1, 0], [2]]), 2);
});

test("difference is symmetric and leaves both matrices unchanged", () => {
  const matrix1 = [
    [1, 2],
    [3, 0],
  ];
  const matrix2 = [[9], [3, 0], [0, 0]];
  const before1 = structuredClone(matrix1);
  const before2 = structuredClone(matrix2);

  assert.equal(difference(matrix1, matrix2), 4);
  assert.equal(difference(matrix2, matrix1), 4);
  assert.deepEqual(matrix1, before1);
  assert.deepEqual(matrix2, before2);
});
