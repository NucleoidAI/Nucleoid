# ARC

## Problem Structure

input_matrix: A rectangular, 2D array in JSON format. It contains numbers (1-9) representing objects, with 0 as empty space.
output_matrix: The result after applying declarative logic to the input_matrix, also a rectangular, 2D JSON array.
input_matrix and output_matrix are representations of the problem's initial and final states.

## Object Definitions

Numbers from 1-9 represent objects.
0 is used for empty spaces.

## Declarative Logic

declarations: Defines the logic that transforms the input_matrix into the output_matrix.

## Input and Output Objects

input_object: A pattern instance from the input_matrix, based on the declarations. It must contain only one instance of the pattern. Any other spaces should be filled with 0s.
output_object: The corresponding instance in the output_matrix. It must also contain only one instance of the pattern, with the remaining space filled with 0s.

## Code Representation

input_code: Represents the input_object.
output_value: Represents the output_object after applying the declarative logic.

## Query Structure

query: A logical intention expressed in nuc language.

## Positioning

In input_object, the x_position is the count from the left, and the y_position is the count from the top.
object_matrix: A zoomed-in, smaller representation of the object.
