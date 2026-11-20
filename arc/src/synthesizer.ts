import * as Matrix from "./lib/Matrix";
import * as visualizer from "./visualizer";
import object from "./object.ts";
import * as nucleoid from "./lib/nucleoid.ts";
import * as milestone from "./lib/milestone.ts";

async function synthesize({ train }) {
  const validated_dataset = [];

  for (const dataset of train.dataset) {
    const synthesized_dataset = { instances: [] };

    for (const instance of dataset.instances) {
      const input_instance_matrix = Matrix.rotate_instance(
        Matrix.move_instance(instance.input_instance_matrix, 1, 1),
        90
      );

      const synthesized_instance_name = instance.instance_name + "s";

      const {
        x: input_x,
        y: input_y,
        width: input_width,
        height: input_height,
        volume: input_volume,
      } = Matrix.measure(input_instance_matrix);

      const synthesized_instance = {
        input_matrix: undefined,
        output_matrix: undefined,
        instance_name: synthesized_instance_name,
        input_value: undefined,
        output_value: undefined,
        input_instance_matrix: input_instance_matrix,
        output_instance_matrix: undefined,
      };

      const input_orientation = await object.orientation({
        train,
        instance: synthesized_instance,
      });

      instance.implementation = await object.implementation({
        train,
        instance: synthesized_instance,
        x: input_x,
        y: input_y,
        width: input_width,
        height: input_height,
        orientation: input_orientation,
        volume: input_volume,
      });

      const canvas_width: number = input_instance_matrix[0].length;
      const canvas_height: number = input_instance_matrix.length;
      const class_declaration = train.class_declaration;
      const declarations = train.declarations;
      const implementations = dataset.instances.map(
        ({ implementation }) => implementation
      );

      synthesized_instance.input_value = await nucleoid.run(
        [
          `canvas_width = ${canvas_width}`,
          `canvas_height = ${canvas_height}`,
          class_declaration,
          instance.implementation,
          `${synthesized_instance_name}`,
        ].join("\n\n")
      );

      synthesized_instance.output_value = await nucleoid.run(
        [
          `canvas_width = ${canvas_width}`,
          `canvas_height = ${canvas_height}`,
          class_declaration,
          ...declarations,
          ...implementations,
          `${synthesized_instance_name}`,
        ].join("\n\n")
      );

      const { output_instance_matrix } =
        await visualizer.output_instance_matrix({
          test_input_matrix: input_instance_matrix,
          train,
          instance: synthesized_instance,
        });

      synthesized_instance.output_instance_matrix = output_instance_matrix;

      const {
        x: output_x,
        y: output_y,
        width: output_width,
        height: output_height,
        volume: output_volume,
      } = Matrix.measure(output_instance_matrix);

      if (
        synthesized_instance.output_value.x === output_x &&
        synthesized_instance.output_value.y === output_y &&
        synthesized_instance.output_value.width === output_width &&
        synthesized_instance.output_value.height === output_height &&
        synthesized_instance.output_value.volume === output_volume
      ) {
        synthesized_dataset.instances.push(synthesized_instance);
      }
    }

    if (synthesized_dataset.instances.length > 0) {
      synthesized_dataset.input_matrix = synthesized_dataset.instances.reduce(
        (acc, instance) => Matrix.merge(acc, instance.input_instance_matrix),
        [[0]]
      );

      synthesized_dataset.output_matrix = synthesized_dataset.instances.reduce(
        (acc, instance) => Matrix.merge(acc, instance.output_instance_matrix),
        [[0]]
      );

      validated_dataset.push(synthesized_dataset);
    }
  }

  milestone.training_dataset_synthesized({ dataset: validated_dataset });

  train.dataset.push(...validated_dataset);
}

export default { synthesize };
