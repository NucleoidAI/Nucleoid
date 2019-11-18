module.exports = class THROW {
  prepare() {}
  run() {
    throw this.exception;
  }
  graph() {}
};
