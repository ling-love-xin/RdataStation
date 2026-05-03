/**
 * 统一错误处理机制
 *
 * 提供标准化的错误类型、错误码和错误处理工具
 */

// ============================================================================
// 错误码定义
// ============================================================================

/** 错误码枚举 */
export enum ErrorCode {
  // 连接相关错误
  CONNECTION_FAILED = 'CONNECTION_FAILED',
  CONNECTION_TIMEOUT = 'CONNECTION_TIMEOUT',
  CONNECTION_REFUSED = 'CONNECTION_REFUSED',
  CONNECTION_LOST = 'CONNECTION_LOST',
  INVALID_CREDENTIALS = 'INVALID_CREDENTIALS',

  // 查询相关错误
  QUERY_FAILED = 'QUERY_FAILED',
  QUERY_TIMEOUT = 'QUERY_TIMEOUT',
  QUERY_CANCELLED = 'QUERY_CANCELLED',
  SYNTAX_ERROR = 'SYNTAX_ERROR',

  // 项目相关错误
  PROJECT_NOT_FOUND = 'PROJECT_NOT_FOUND',
  PROJECT_LOAD_FAILED = 'PROJECT_LOAD_FAILED',
  PROJECT_SAVE_FAILED = 'PROJECT_SAVE_FAILED',

  // 文件相关错误
  FILE_NOT_FOUND = 'FILE_NOT_FOUND',
  FILE_READ_FAILED = 'FILE_READ_FAILED',
  FILE_WRITE_FAILED = 'FILE_WRITE_FAILED',

  // 权限相关错误
  PERMISSION_DENIED = 'PERMISSION_DENIED',
  ACCESS_DENIED = 'ACCESS_DENIED',

  // 系统相关错误
  INTERNAL_ERROR = 'INTERNAL_ERROR',
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}

// ============================================================================
// 错误类型定义
// ============================================================================

/** 应用错误 */
export class AppError extends Error {
  /** 错误码 */
  public readonly code: ErrorCode

  /** 错误详情 */
  public readonly details?: unknown

  /** 原始错误 */
  public readonly cause?: Error

  /** 时间戳 */
  public readonly timestamp: Date

  constructor(
    code: ErrorCode,
    message: string,
    options?: {
      details?: unknown
      cause?: Error
    }
  ) {
    super(message)
    this.name = 'AppError'
    this.code = code
    this.details = options?.details
    this.cause = options?.cause
    this.timestamp = new Date()

    // 保持正确的原型链
    Object.setPrototypeOf(this, AppError.prototype)
  }

  /**
   * 转换为可序列化的对象
   */
  toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      details: this.details,
      timestamp: this.timestamp.toISOString(),
      stack: this.stack,
    }
  }

  /**
   * 获取用户友好的错误消息
   */
  getUserMessage(): string {
    switch (this.code) {
      case ErrorCode.CONNECTION_FAILED:
        return '连接失败，请检查网络或数据库服务状态'
      case ErrorCode.CONNECTION_TIMEOUT:
        return '连接超时，请检查网络设置'
      case ErrorCode.CONNECTION_REFUSED:
        return '连接被拒绝，请检查数据库配置'
      case ErrorCode.INVALID_CREDENTIALS:
        return '用户名或密码错误'
      case ErrorCode.QUERY_FAILED:
        return '查询执行失败'
      case ErrorCode.QUERY_TIMEOUT:
        return '查询超时，请优化 SQL 或增加超时时间'
      case ErrorCode.SYNTAX_ERROR:
        return 'SQL 语法错误，请检查查询语句'
      case ErrorCode.PROJECT_NOT_FOUND:
        return '项目不存在'
      case ErrorCode.FILE_NOT_FOUND:
        return '文件不存在'
      case ErrorCode.PERMISSION_DENIED:
        return '权限不足，无法执行此操作'
      default:
        return this.message || '发生未知错误'
    }
  }
}

// ============================================================================
// Result 类型
// ============================================================================

/** 成功结果 */
export interface Success<T> {
  ok: true
  value: T
}

/** 失败结果 */
export interface Failure<E = AppError> {
  ok: false
  error: E
}

/** Result 类型 */
export type Result<T, E = AppError> = Success<T> | Failure<E>

/**
 * 创建成功结果
 */
export function success<T>(value: T): Success<T> {
  return { ok: true, value }
}

/**
 * 创建失败结果
 */
export function failure<E = AppError>(error: E): Failure<E> {
  return { ok: false, error }
}

/**
 * 判断是否为成功结果
 */
export function isSuccess<T, E>(result: Result<T, E>): result is Success<T> {
  return result.ok
}

/**
 * 判断是否为失败结果
 */
export function isFailure<T, E>(result: Result<T, E>): result is Failure<E> {
  return !result.ok
}

// ============================================================================
// 错误处理工具函数
// ============================================================================

/**
 * 安全执行异步函数，返回 Result 类型
 *
 * @param fn 异步函数
 * @returns Result<T, AppError>
 */
export async function safeAsync<T>(
  fn: () => Promise<T>
): Promise<Result<T, AppError>> {
  try {
    const value = await fn()
    return success(value)
  } catch (error) {
    return failure(toAppError(error))
  }
}

/**
 * 将未知错误转换为 AppError
 */
export function toAppError(error: unknown): AppError {
  if (error instanceof AppError) {
    return error
  }

  if (error instanceof Error) {
    return new AppError(ErrorCode.UNKNOWN_ERROR, error.message, { cause: error })
  }

  if (typeof error === 'string') {
    return new AppError(ErrorCode.UNKNOWN_ERROR, error)
  }

  return new AppError(ErrorCode.INTERNAL_ERROR, '内部错误')
}

/**
 * 安全执行同步函数，返回 Result 类型
 *
 * @param fn 同步函数
 * @returns Result<T, AppError>
 */
export function safeSync<T>(
  fn: () => T
): Result<T, AppError> {
  try {
    const value = fn()
    return success(value)
  } catch (error) {
    return failure(toAppError(error))
  }
}

/**
 * 解包 Result，成功返回值，失败抛异常
 */
export function unwrap<T, E = AppError>(result: Result<T, E>): T {
  if (result.ok) {
    return result.value
  }
  throw result.error
}

/**
 * 解包 Result，成功返回值，失败返回默认值
 */
export function unwrapOr<T, E>(result: Result<T, E>, defaultValue: T): T {
  if (result.ok) {
    return result.value
  }
  return defaultValue
}

/**
 * 解包 Result，成功返回值，失败执行回调
 */
export function unwrapOrElse<T, E>(
  result: Result<T, E>,
  fn: (error: E) => T
): T {
  if (result.ok) {
    return result.value
  }
  return fn(result.error)
}

// ============================================================================
// 错误日志
// ============================================================================

/** 日志级别 */
export enum LogLevel {
  DEBUG = 'DEBUG',
  INFO = 'INFO',
  WARN = 'WARN',
  ERROR = 'ERROR',
}

/**
 * 记录错误日志
 */
export function logError(
  error: AppError,
  level: LogLevel = LogLevel.ERROR,
  context?: Record<string, unknown>
): void {
  const logEntry = {
    level,
    code: error.code,
    message: error.message,
    timestamp: error.timestamp.toISOString(),
    context,
  }

  switch (level) {
    case LogLevel.DEBUG:
      console.debug('[AppError]', logEntry)
      break
    case LogLevel.INFO:
      console.info('[AppError]', logEntry)
      break
    case LogLevel.WARN:
      console.warn('[AppError]', logEntry)
      break
    case LogLevel.ERROR:
      console.error('[AppError]', logEntry)
      break
  }
}
