export const tv = <T>(condition: unknown, value?: T) =>
  condition ? value : void 0

export const fv = <T>(condition: unknown, value?: T) => tv(!condition, value)

export const run =
  <A extends unknown[], V>(fn: (...args: A) => V, ...args: A) =>
  (): V =>
    fn(...args)