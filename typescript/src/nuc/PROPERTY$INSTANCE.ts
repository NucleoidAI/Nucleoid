import PROPERTY from "./PROPERTY";

class PROPERTY$INSTANCE extends PROPERTY {
  before(scope: { $instance: any }): void {
    const $instance: any = scope.$instance;

    if (!$instance) {
      throw "Declaration missing instance in scope";
    }

    const tokens = (this.value as any).tokens;
    tokens.traverse((node: any) => {
      const identifiers = [node.walk()].flat(Infinity);
      for (const identifier of identifiers) {
        if ($instance.class.name.toString() === identifier.first.toString()) {
          identifier.first = $instance.resolve();
        }
      }
    });

    super.before(scope);
  }
}

export default PROPERTY$INSTANCE;
