import LET from "./LET";

class LET$INSTANCE extends LET {
  before(scope: any): void {
    this.value.tokens.traverse((node: any) => {
      const identifiers: any[] = [node.walk()].flat(Infinity);

      for (const identifier of identifiers) {
        if (identifier.first.toString() === this.class.name.toString()) {
          identifier.first = this.instance.resolve();
        }
      }
    });

    super.before(scope);
  }
}

module.exports = LET$INSTANCE;
