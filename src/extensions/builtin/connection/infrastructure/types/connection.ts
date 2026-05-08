/**
 * 连接类型定义
 */

import type {
  Connection,
  RecentConnection,
  ProjectConnection,
  CreateProjectConnectionInput,
} from '../../types/connection'

export type { Connection, RecentConnection, ProjectConnection, CreateProjectConnectionInput }

// 项目连接响应
export interface ProjectConnectionResponse {
  id: string
  name: string
  db_type: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
  created_at: string
  updated_at: string
}

// 创建项目连接输入
export interface CreateConnectionInput {
  project_path: string
  name: string
  db_type: string
  host: string
  port?: number
  database?: string
  username?: string
  password?: string
  properties?: Record<string, unknown>
}
