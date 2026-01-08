/**
 * Functional Programming Utilities
 * Provides composable functions for cleaner, more maintainable code
 */

import type { Result, SError } from '@gen/bindings'

/**
 * Unwraps a Result type, throwing an error if it's an error result
 */
export function unwrapResult<T>(result: Result<T, SError>): T {
  if (result.status === 'ok') {
    return result.data
  }
  throw new Error(result.error.message || 'Unknown error')
}

/**
 * Pattern matching for Result types
 */
export function match<T, E, R>(
  result: Result<T, E>,
  onOk: (data: T) => R,
  onError: (error: E) => R,
): R {
  if (result.status === 'ok') {
    return onOk(result.data)
  }
  return onError(result.error)
}

/**
 * Maps over a Result value if it's successful
 */
export function mapResult<T, U, E>(
  result: Result<T, E>,
  fn: (data: T) => U,
): Result<U, E> {
  if (result.status === 'ok') {
    return { status: 'ok', data: fn(result.data) }
  }
  return result as Result<U, E>
}

/**
 * Composes functions from left to right
 */
export function pipe<A>(value: A): A
export function pipe<A, B>(value: A, fn1: (a: A) => B): B
export function pipe<A, B, C>(value: A, fn1: (a: A) => B, fn2: (b: B) => C): C
export function pipe<A, B, C, D>(
  value: A,
  fn1: (a: A) => B,
  fn2: (b: B) => C,
  fn3: (c: C) => D,
): D
export function pipe(value: any, ...fns: Array<(arg: any) => any>): any {
  return fns.reduce((acc, fn) => fn(acc), value)
}

/**
 * Composes async functions from left to right
 */
export async function asyncPipe<A>(value: A): Promise<A>
export async function asyncPipe<A, B>(
  value: A,
  fn1: (a: A) => Promise<B>,
): Promise<B>
export async function asyncPipe<A, B, C>(
  value: A,
  fn1: (a: A) => Promise<B>,
  fn2: (b: B) => Promise<C>,
): Promise<C>
export async function asyncPipe<A, B, C, D>(
  value: A,
  fn1: (a: A) => Promise<B>,
  fn2: (b: B) => Promise<C>,
  fn3: (c: C) => Promise<D>,
): Promise<D>
export async function asyncPipe(
  value: any,
  ...fns: Array<(arg: any) => Promise<any>>
): Promise<any> {
  let result = value
  for (const fn of fns) {
    result = await fn(result)
  }
  return result
}

/**
 * Creates a function that handles errors gracefully
 */
export function tryCatch<T, E = Error>(
  fn: () => T,
  onError: (error: unknown) => E,
): T | E {
  try {
    return fn()
  } catch (error) {
    return onError(error)
  }
}

/**
 * Async version of tryCatch
 */
export async function asyncTryCatch<T, E = Error>(
  fn: () => Promise<T>,
  onError: (error: unknown) => E,
): Promise<T | E> {
  try {
    return await fn()
  } catch (error) {
    return onError(error)
  }
}

/**
 * Delays execution
 */
export function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * Retries a function multiple times with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: {
    maxAttempts?: number
    delayMs?: number
    backoff?: boolean
  } = {},
): Promise<T> {
  const { maxAttempts = 3, delayMs = 1000, backoff = true } = options
  let lastError: unknown

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn()
    } catch (error) {
      lastError = error
      if (attempt < maxAttempts) {
        const waitTime = backoff ? delayMs * Math.pow(2, attempt - 1) : delayMs
        await delay(waitTime)
      }
    }
  }

  throw lastError
}

/**
 * Debounces a function call
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  ms: number,
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | undefined

  return function (...args: Parameters<T>) {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
    timeoutId = setTimeout(() => fn(...args), ms)
  }
}

/**
 * Throttles a function call
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  ms: number,
): (...args: Parameters<T>) => void {
  let lastCall = 0

  return function (...args: Parameters<T>) {
    const now = Date.now()
    if (now - lastCall >= ms) {
      lastCall = now
      fn(...args)
    }
  }
}

/**
 * Memoizes a function result
 */
export function memoize<T extends (...args: any[]) => any>(
  fn: T,
): T & { cache: Map<string, ReturnType<T>> } {
  const cache = new Map<string, ReturnType<T>>()

  const memoized = function (...args: Parameters<T>): ReturnType<T> {
    const key = JSON.stringify(args)
    if (cache.has(key)) {
      return cache.get(key)!
    }
    const result = fn(...args)
    cache.set(key, result)
    return result
  } as T & { cache: Map<string, ReturnType<T>> }

  memoized.cache = cache
  return memoized
}

/**
 * Identity function - returns the input unchanged
 */
export function identity<T>(value: T): T {
  return value
}

/**
 * Always returns the same value
 */
export function constant<T>(value: T): () => T {
  return () => value
}

/**
 * Checks if a value is not null or undefined
 */
export function isNotNil<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined
}

/**
 * Filters out null and undefined values
 */
export function compact<T>(array: Array<T | null | undefined>): T[] {
  return array.filter(isNotNil)
}
