import { invoke } from '@tauri-apps/api/core'

export interface ProjectInfo {
  id: string
  name: string
  description?: string
  path: {
    type: 'Local' | 'Remote'
    path?: string
    url?: string
    project_id?: string
  }
  status: string
  created_at: string
  updated_at: string
  last_opened_at?: string
  version: string
}

export interface CreateProjectInput {
  name: string
  path: string
  description?: string
}

export class ProjectService {
  /**
   * 获取最近项目列表
   * @param limit 返回数量限制，默认 10
   */
  static async getRecentProjects(limit: number = 10): Promise<ProjectInfo[]> {
    return invoke<ProjectInfo[]>('get_recent_projects', { limit })
  }

  /**
   * 根据 ID 打开项目
   * @param id 项目 ID
   */
  static async openProjectById(id: string): Promise<ProjectInfo> {
    return invoke<ProjectInfo>('open_project_by_id', { id })
  }

  /**
   * 根据路径打开项目
   * @param path 项目路径
   */
  static async openProjectByPath(path: string): Promise<ProjectInfo> {
    return invoke<ProjectInfo>('open_project_by_path', { path })
  }

  /**
   * 创建并保存项目到全局数据库
   * @param input 项目创建参数
   */
  static async createAndSaveProject(input: CreateProjectInput): Promise<ProjectInfo> {
    console.log('ProjectService.createAndSaveProject 调用:', input)
    const result = await invoke<ProjectInfo>('create_and_save_project', { input })
    console.log('ProjectService.createAndSaveProject 返回:', result)
    return result
  }

  /**
   * 添加到最近项目（更新最后打开时间）
   * @param projectId 项目 ID
   */
  static async addRecentProject(projectId: string): Promise<void> {
    return invoke<void>('add_recent_project', { projectId })
  }
}
