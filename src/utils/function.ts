import { Result, SError } from '@gen/bindings'
import { ur } from './result'
import { ett } from './error'

export function createSetter<
  A extends Parameters<T>,
  R,
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  T extends (...args: any[]) => Promise<Result<R, SError>>,
>(fn: T, setter: (value: R) => void): (...args: A) => Promise<void> {
  return (...args: A) =>
    fn(...args)
      .then(ur)
      .then(setter)
      .catch(ett)
}
