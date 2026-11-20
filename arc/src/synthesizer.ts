import * as Matrix from "./lib/Matrix";
import * as visualizer from "./visualizer";
import object from "./object.ts";
import * as nucleoid from "./lib/nucleoid.ts";

async function synthesize({ train }) {
  const validated_dataset = [];

  for (const dataset of train.dataset) {
    const synthesized_dataset = { instances: [] };

    for (const instance of dataset.instances) {
      const input_instance_matrix = Matrix.move(
        Matrix.rotate(instance.input_instance_matrix, 90),
        1,
        1
      );

      const canvas_width: number = input_instance_matrix[0].length;
      const canvas_height: number = input_instance_matrix.length;
      const class_declaration = train.class_declaration;
      const declarations = train.declarations;
      const implementations = dataset.instances.map(
        ({ implementation }) => implementation
      );

      const output_value = await nucleoid.run(
        [
          `canvas_width = ${canvas_width}`,
          `canvas_height = ${canvas_height}`,
          class_declaration,
          ...declarations,
          ...implementations,
          `${instance.instance_name}.orientation += 90`,
          `${instance.instance_name}.x += 1`,
          `${instance.instance_name}.y += 1`,
          `${instance.instance_name}`,
        ].join("\n\n")
      );

      const synthesized_instance_name = instance.instance_name + "s";
      output_value.id = synthesized_instance_name;

      const synthesized_instance = {
        input_matrix: undefined,
        output_matrix: undefined,
        instance_name: synthesized_instance_name,
        input_value: { ...instance.input_value, id: synthesized_instance_name },
        output_value,
        input_instance_matrix: input_instance_matrix,
        output_instance_matrix: undefined,
      };

      const { output_instance_matrix } =
        await visualizer.output_instance_matrix({
          test_input_matrix: input_instance_matrix,
          train,
          instance: synthesized_instance,
        });

      synthesized_instance.output_instance_matrix = output_instance_matrix;

      const orientation = await object.orientation({
        train,
        instance: synthesized_instance,
      });

      const { x, y, width, height, volume } = Matrix.measure(
        output_instance_matrix
      );

      instance.implementation = await object.implementation({
        train,
        instance: synthesized_instance,
        x,
        y,
        width,
        height,
        orientation,
        volume,
      });

      if (
        output_value.x === x &&
        output_value.y === y &&
        output_value.width === width &&
        output_value.height === height &&
        output_value.volume === volume &&
        output_value.orientation === orientation
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

  train.dataset.push(...validated_dataset);
}

export default { synthesize };
