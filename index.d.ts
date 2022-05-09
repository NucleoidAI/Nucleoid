declare module "nucleoidjs" {
  export abstract class Prototype {
    static filter(param: (p) => any);
  }

  function nucleoid();
  export default nucleoid;
}
