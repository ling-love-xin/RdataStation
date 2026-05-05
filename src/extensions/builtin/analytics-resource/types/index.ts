// 分析资源类型定义

export type ResourceType = 'connection' | 'table' | 'file';

export type ResourceScope = 'global' | 'project' | 'session';

export interface AnalyticsResource {
  id: string;
  resource_type: ResourceType;
  name: string;
  alias?: string;
  config: Record<string, any>;
  scope: ResourceScope;
  row_count?: number;
  column_count?: number;
  file_size?: number;
  version: number;
  parent_version_id?: string;
  parent_resource_id?: string;
  source_query?: string;
  created_at: string;
  updated_at: string;
  created_by?: string;
  deleted_at?: string;
}

export interface AnalyticsFolder {
  id: string;
  name: string;
  scope: ResourceScope;
  parent_folder_id?: string;
  sort_order: number;
  color?: string;
  icon?: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

export interface AnalyticsTag {
  id: string;
  name: string;
  color?: string;
  icon?: string;
  scope: ResourceScope;
  created_at: string;
  deleted_at?: string;
}

export interface AnalyticsRecycleItem {
  id: string;
  resource_id: string;
  resource_type: ResourceType;
  resource_name: string;
  resource_data: Record<string, any>;
  deleted_by?: string;
  deleted_at: string;
}

export interface CreateResourceRequest {
  resource_type: ResourceType;
  name: string;
  alias?: string;
  config: Record<string, any>;
  scope: ResourceScope;
  row_count?: number;
  column_count?: number;
  file_size?: number;
  parent_resource_id?: string;
  source_query?: string;
}

export interface CreateFolderRequest {
  name: string;
  scope: ResourceScope;
  parent_folder_id?: string;
  color?: string;
  icon?: string;
}

export interface CreateTagRequest {
  name: string;
  color?: string;
  icon?: string;
  scope: ResourceScope;
}

// ==================== Pagination & Sorting ====================

export type SortOrder = 'asc' | 'desc';

export type SortField = 'name' | 'created_at' | 'updated_at' | 'row_count' | 'file_size';

export interface PaginationInput {
  page?: number;
  pageSize?: number;
}

export interface SortInput {
  sortBy?: SortField;
  sortOrder?: SortOrder;
}

export interface ListResourcesInput {
  scope?: string;
  resource_type?: string;
  folder_id?: string;
  search?: string;
  pagination?: PaginationInput;
  sort?: SortInput;
}

export interface ListResourcesOutput {
  items: AnalyticsResource[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface ListFoldersInput {
  scope?: string;
  parent_folder_id?: string;
}

export interface ListTagsInput {
  scope?: string;
}
