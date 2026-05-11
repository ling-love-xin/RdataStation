/**
 * DBeaver风格数据库导航栏示例数据
 */

export interface NavigatorNode {
  id: string
  name: string
  type:
    | 'connection'
    | 'catalog'
    | 'schema'
    | 'table'
    | 'view'
    | 'procedure'
    | 'function'
    | 'column'
    | 'index'
    | 'trigger'
    | 'folder'
  children?: NavigatorNode[]
  isLeaf?: boolean
  data?: any
}

export interface Connection {
  id: string
  name: string
  type: string
  host: string
  database: string
}

// 示例连接列表
export const mockConnections: Connection[] = [
  { id: 'conn_1', name: '本地 MySQL', type: 'mysql', host: 'localhost:3306', database: 'test_db' },
  {
    id: 'conn_2',
    name: '生产 PostgreSQL',
    type: 'postgres',
    host: 'prod.db.com:5432',
    database: 'production',
  },
  { id: 'conn_3', name: '开发 SQLite', type: 'sqlite', host: '', database: 'dev.db' },
]

// 示例导航节点数据
export const mockNavigatorNodes: NavigatorNode[] = [
  {
    id: 'db_1',
    name: 'test_db',
    type: 'catalog',
    children: [
      {
        id: 'schema_1',
        name: 'public',
        type: 'schema',
        children: [
          {
            id: 'tables_folder',
            name: '表',
            type: 'folder',
            children: [
              {
                id: 'table_1',
                name: 'users',
                type: 'table',
                children: [
                  {
                    id: 'col_1',
                    name: 'id',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'INT', nullable: false, isPrimaryKey: true },
                  },
                  {
                    id: 'col_2',
                    name: 'username',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'VARCHAR(50)', nullable: false },
                  },
                  {
                    id: 'col_3',
                    name: 'email',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'VARCHAR(100)', nullable: false },
                  },
                  {
                    id: 'col_4',
                    name: 'created_at',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'TIMESTAMP', nullable: false },
                  },
                ],
              },
              {
                id: 'table_2',
                name: 'orders',
                type: 'table',
                children: [
                  {
                    id: 'col_5',
                    name: 'id',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'INT', nullable: false, isPrimaryKey: true },
                  },
                  {
                    id: 'col_6',
                    name: 'user_id',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'INT', nullable: false, isForeignKey: true },
                  },
                  {
                    id: 'col_7',
                    name: 'total_amount',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'DECIMAL(10,2)', nullable: false },
                  },
                  {
                    id: 'col_8',
                    name: 'status',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'VARCHAR(20)', nullable: false },
                  },
                ],
              },
              {
                id: 'table_3',
                name: 'products',
                type: 'table',
                children: [
                  {
                    id: 'col_9',
                    name: 'id',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'INT', nullable: false, isPrimaryKey: true },
                  },
                  {
                    id: 'col_10',
                    name: 'name',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'VARCHAR(200)', nullable: false },
                  },
                  {
                    id: 'col_11',
                    name: 'price',
                    type: 'column',
                    isLeaf: true,
                    data: { dataType: 'DECIMAL(10,2)', nullable: false },
                  },
                ],
              },
            ],
          },
          {
            id: 'views_folder',
            name: '视图',
            type: 'folder',
            children: [
              {
                id: 'view_1',
                name: 'user_orders_view',
                type: 'view',
                isLeaf: true,
              },
              {
                id: 'view_2',
                name: 'product_summary',
                type: 'view',
                isLeaf: true,
              },
            ],
          },
          {
            id: 'procedures_folder',
            name: '存储过程',
            type: 'folder',
            children: [
              {
                id: 'proc_1',
                name: 'sp_get_user_orders',
                type: 'procedure',
                isLeaf: true,
              },
              {
                id: 'proc_2',
                name: 'sp_update_inventory',
                type: 'procedure',
                isLeaf: true,
              },
            ],
          },
          {
            id: 'functions_folder',
            name: '函数',
            type: 'folder',
            children: [
              {
                id: 'func_1',
                name: 'fn_calculate_total',
                type: 'function',
                isLeaf: true,
              },
            ],
          },
        ],
      },
    ],
  },
]

// 模拟异步加载子节点
export async function loadNavigatorChildren(parentId: string): Promise<NavigatorNode[]> {
  // 模拟网络延迟
  await new Promise(resolve => setTimeout(resolve, 300))

  // 这里可以根据 parentId 返回对应的子节点
  // 目前返回空数组，表示已加载完成
  return []
}
