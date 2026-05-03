/**
 * SQL 方言高亮服务
 * 根据数据库类型动态调整 Monaco Editor 的语法高亮规则
 */

import * as monaco from 'monaco-editor'

export type SqlDialect = 
  | 'generic'
  | 'mysql'
  | 'postgres'
  | 'sqlite'
  | 'duckdb'
  | 'mssql'
  | 'oracle'
  | 'snowflake'
  | 'bigquery'
  | 'redshift'

// 各数据库特有的关键字
const DIALECT_KEYWORDS: Record<SqlDialect, string[]> = {
  generic: [],
  mysql: [
    'ENGINE', 'CHARSET', 'COLLATE', 'AUTO_INCREMENT', 'UNSIGNED',
    'TINYINT', 'MEDIUMINT', 'YEAR', 'DATETIME', 'ENUM', 'SET',
    'SHOW', 'USE', 'EXPLAIN', 'DESCRIBE', 'OPTIMIZE', 'REPAIR',
    'ANALYZE', 'CHECK', 'REPLACE', 'IGNORE', 'DELAYED',
    'STRAIGHT_JOIN', 'SQL_SMALL_RESULT', 'SQL_BIG_RESULT',
    'SQL_BUFFER_RESULT', 'SQL_CACHE', 'SQL_NO_CACHE'
  ],
  postgres: [
    'SERIAL', 'BIGSERIAL', 'UUID', 'JSONB', 'XML', 'INET',
    'CIDR', 'MACADDR', 'BIT', 'VARBIT', 'TSVECTOR', 'TSQUERY',
    'RETURNING', 'CONFLICT', 'DO', 'NOTHING', 'ILIKE',
    'SIMILAR', 'TO', 'DISTINCT', 'ON', 'LATERAL', 'CROSS',
    'FULL', 'OUTER', 'INNER', 'JOIN', 'NATURAL', 'USING',
    'WINDOW', 'PARTITION', 'ORDER', 'BY', 'RANGE', 'ROWS',
    'BETWEEN', 'UNBOUNDED', 'PRECEDING', 'FOLLOWING', 'CURRENT',
    'ROW', 'GROUP', 'SETS', 'CUBE', 'ROLLUP', 'GROUPING',
    'WITH', 'TIME', 'ZONE', 'AT', 'OVERLAPS', 'IS', 'NULL',
    'TRUE', 'FALSE', 'UNKNOWN', 'CASE', 'WHEN', 'THEN', 'ELSE',
    'END', 'CAST', 'TRY_CAST', 'EXTRACT', 'OVERLAY', 'POSITION',
    'SUBSTRING', 'TRIM', 'TRANSLATE', 'CONVERT', 'TREAT'
  ],
  sqlite: [
    'AUTOINCREMENT', 'BLOB', 'TEXT', 'REAL', 'NUMERIC',
    'INTEGER', 'WITHOUT', 'ROWID', 'RECURSIVE', 'INDEXED',
    'BY', 'STRICT', 'MATERIALIZED', 'FILTER', 'OVER',
    'PARTITION', 'ORDER', 'GROUPS', 'RANGE', 'ROWS',
    'EXCLUDE', 'TIES', 'OTHERS', 'NULLS', 'FIRST', 'LAST',
    'ESCAPE', 'GLOB', 'REGEXP', 'LIKE'
  ],
  duckdb: [
    'STRUCT', 'MAP', 'UNION', 'LIST', 'ARRAY', 'LARGEINT',
    'HUGEINT', 'UBIGINT', 'UINTEGER', 'USMALLINT', 'UTINYINT',
    'TIMESTAMP_S', 'TIMESTAMP_MS', 'TIMESTAMP_NS', 'DATE',
    'INTERVAL', 'UUID', 'BIT', 'BLOB', 'VARCHAR', 'VARINT',
    'PIVOT', 'UNPIVOT', 'SAMPLE', 'TABLESAMPLE', 'USING',
    'GROUP', 'ALL', 'CUBE', 'ROLLUP', 'GROUPING', 'SETS',
    'QUALIFY', 'WINDOW', 'PARTITION', 'ORDER', 'ROWS',
    'RANGE', 'GROUPS', 'EXCLUDE', 'TIES', 'OTHERS',
    'NULLS', 'FIRST', 'LAST', 'RESPECT', 'IGNORE', 'NULLS'
  ],
  mssql: [
    'NVARCHAR', 'NCHAR', 'DATETIME2', 'DATETIMEOFFSET',
    'SMALLDATETIME', 'MONEY', 'SMALLMONEY', 'UNIQUEIDENTIFIER',
    'HIERARCHYID', 'GEOMETRY', 'GEOGRAPHY', 'SQL_VARIANT',
    'TABLE', 'TOP', 'PERCENT', 'WITH', 'TIES', 'OUTPUT',
    'INSERTED', 'DELETED', 'MERGE', 'WHEN', 'MATCHED',
    'NOT', 'FOR', 'REPLICATION', 'CHECK', 'NOCHECK',
    'ENCRYPTION', 'SCHEMABINDING', 'VIEW_METADATA',
    'EXECUTE', 'AS', 'OWNER', 'CALLER', 'SELF'
  ],
  oracle: [
    'VARCHAR2', 'NVARCHAR2', 'NUMBER', 'FLOAT', 'LONG',
    'LONG', 'RAW', 'BFILE', 'ROWID', 'UROWID',
    'TIMESTAMP', 'INTERVAL', 'YEAR', 'MONTH', 'DAY',
    'SECOND', 'TIMEZONE', 'REGION', 'ABBR', 'CLOB',
    'NCLOB', 'BLOB', 'XMLTYPE', 'SDO_GEOMETRY',
    'CONNECT', 'PRIOR', 'LEVEL', 'NOCYCLE', 'ORDER',
    'SIBLINGS', 'MODEL', 'DIMENSION', 'MEASURES',
    'RULES', 'UPSERT', 'ITERATE', 'RETURN', 'UPDATED',
    'ROWS', 'ONLY', 'PARTITION', 'BY', 'RANGE',
    'UNBOUNDED', 'PRECEDING', 'FOLLOWING', 'CURRENT', 'ROW'
  ],
  snowflake: [
    'VARIANT', 'OBJECT', 'ARRAY', 'GEOGRAPHY', 'GEOMETRY',
    'TIMESTAMP_LTZ', 'TIMESTAMP_NTZ', 'TIMESTAMP_TZ',
    'CLUSTER', 'KEY', 'COPY', 'INTO', 'FROM', 'FILE_FORMAT',
    'FORMAT_NAME', 'STAGE', 'WAREHOUSE', 'DATABASE', 'SCHEMA',
    'TABLE', 'VIEW', 'STREAM', 'TASK', 'PIPE', 'FUNCTION',
    'PROCEDURE', 'SEQUENCE', 'SHARE', 'ROLE', 'USER',
    'GRANT', 'REVOKE', 'ALTER', 'MODIFY', 'SET', 'UNSET',
    'RENAME', 'SWAP', 'CLONE', 'DROP', 'TRUNCATE', 'PURGE'
  ],
  bigquery: [
    'STRING', 'BYTES', 'INT64', 'FLOAT64', 'BOOL', 'NUMERIC',
    'BIGNUMERIC', 'ARRAY', 'STRUCT', 'GEOGRAPHY', 'JSON',
    'INTERVAL', 'RANGE', 'PARTITION', 'CLUSTER', 'BY',
    'OPTIONS', 'AS', 'TABLESAMPLE', 'SYSTEM', 'BERNOULLI',
    'RESERVOIR', 'ROWS', 'RANGE', 'GROUPS', 'QUALIFY',
    'WINDOW', 'PARTITION', 'ORDER', 'ROWS', 'RANGE',
    'GROUPS', 'EXCLUDE', 'TIES', 'OTHERS', 'NULLS',
    'FIRST', 'LAST', 'RESPECT', 'IGNORE', 'NULLS',
    'SAFE', 'CAST', 'LIKE', 'REGEXP_CONTAINS', 'REGEXP_EXTRACT'
  ],
  redshift: [
    'SMALLINT', 'INTEGER', 'BIGINT', 'DECIMAL', 'REAL',
    'DOUBLE', 'PRECISION', 'BOOLEAN', 'CHAR', 'VARCHAR',
    'DATE', 'TIMESTAMP', 'TIMESTAMPTZ', 'TIME', 'TIMETZ',
    'INTERVAL', 'SUPER', 'STRUCT', 'GEOMETRY', 'GEOGRAPHY',
    'VARBYTE', 'DISTKEY', 'SORTKEY', 'COMPOUND', 'INTERLEAVED',
    'DISTSTYLE', 'EVEN', 'ALL', 'AUTO', 'ENCODE', 'RAW',
    'AZ64', 'BYTEDICT', 'DELTA', 'DELTA32K', 'LZO', 'MOSTLY8',
    'MOSTLY16', 'MOSTLY32', 'RUNLENGTH', 'TEXT255', 'TEXT32K',
    'ZSTD', 'COPY', 'FROM', 'TO', 'CREDENTIALS', 'IAM_ROLE',
    'REGION', 'ACCESS_KEY_ID', 'SECRET_ACCESS_KEY', 'SESSION_TOKEN'
  ]
}

// 各数据库特有的函数
const DIALECT_FUNCTIONS: Record<SqlDialect, string[]> = {
  generic: ['COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'COALESCE', 'NULLIF', 'CASE', 'CAST'],
  mysql: [
    'IFNULL', 'IF', 'FIELD', 'FIND_IN_SET', 'GROUP_CONCAT',
    'CONCAT_WS', 'DATE_FORMAT', 'STR_TO_DATE', 'UNIX_TIMESTAMP',
    'FROM_UNIXTIME', 'NOW', 'CURDATE', 'CURTIME', 'UTC_DATE',
    'UTC_TIME', 'UTC_TIMESTAMP', 'LAST_INSERT_ID', 'ROW_COUNT',
    'FOUND_ROWS', 'CONNECTION_ID', 'VERSION', 'DATABASE', 'USER',
    'CURRENT_USER', 'CHARSET', 'COLLATION', 'BENCHMARK', 'SLEEP'
  ],
  postgres: [
    'COALESCE', 'NULLIF', 'GREATEST', 'LEAST', 'ARRAY_AGG',
    'STRING_AGG', 'JSON_AGG', 'JSONB_AGG', 'JSON_OBJECT',
    'JSONB_OBJECT', 'ROW_TO_JSON', 'JSONB_TO_RECORD',
    'XMLAGG', 'BOOL_AND', 'BOOL_OR', 'EVERY', 'ANY', 'ALL',
    'SOME', 'ARRAY', 'CARDINALITY', 'ARRAY_LENGTH', 'ARRAY_DIMS',
    'ARRAY_UPPER', 'ARRAY_LOWER', 'GENERATE_SERIES', 'RANDOM',
    'SETSEED', 'CURRENT_DATE', 'CURRENT_TIME', 'CURRENT_TIMESTAMP',
    'LOCALTIME', 'LOCALTIMESTAMP', 'NOW', 'STATEMENT_TIMESTAMP',
    'TRANSACTION_TIMESTAMP', 'CLOCK_TIMESTAMP', 'AGE', 'DATE_PART',
    'DATE_TRUNC', 'EXTRACT', 'JUSTIFY_DAYS', 'JUSTIFY_HOURS',
    'JUSTIFY_INTERVAL', 'MAKE_DATE', 'MAKE_INTERVAL', 'MAKE_TIME',
    'MAKE_TIMESTAMP', 'MAKE_TIMESTAMPTZ'
  ],
  sqlite: [
    'ABS', 'CHANGES', 'COALESCE', 'HEX', 'IFNULL', 'INSTR',
    'LAST_INSERT_ROWID', 'LENGTH', 'LIKELIHOOD', 'LIKELY',
    'LOAD_EXTENSION', 'LOWER', 'LTRIM', 'MAX', 'MIN', 'NULLIF',
    'PRINTF', 'QUOTE', 'RANDOM', 'RANDOMBLOB', 'REPLACE',
    'ROUND', 'RTRIM', 'SOUNDEX', 'SQLITE_COMPILEOPTION_GET',
    'SQLITE_COMPILEOPTION_USED', 'SQLITE_OFFSET', 'SQLITE_SOURCE_ID',
    'SQLITE_VERSION', 'SUBSTR', 'SUM', 'TOTAL', 'TOTAL_CHANGES',
    'TRIM', 'TYPEOF', 'UNICODE', 'UNLIKELY', 'UPPER', 'ZEROBLOB'
  ],
  duckdb: [
    'ARRAY_AGG', 'LIST_AGG', 'STRING_AGG', 'HISTOGRAM',
    'APPROX_COUNT_DISTINCT', 'APPROX_QUANTILE', 'MEDIAN',
    'MODE', 'QUANTILE', 'QUANTILE_CONT', 'QUANTILE_DISC',
    'STDDEV', 'STDDEV_POP', 'STDDEV_SAMP', 'VAR_POP',
    'VAR_SAMP', 'VARIANCE', 'CORR', 'COVAR_POP', 'COVAR_SAMP',
    'REGR_AVGX', 'REGR_AVGY', 'REGR_COUNT', 'REGR_INTERCEPT',
    'REGR_R2', 'REGR_SLOPE', 'REGR_SXX', 'REGR_SXY', 'REGR_SYY',
    'BIT_AND', 'BIT_OR', 'BIT_XOR', 'BOOL_AND', 'BOOL_OR',
    'EVERY', 'ANY_VALUE', 'FIRST', 'LAST', 'ARBITRARY',
    'PIVOT', 'UNPIVOT', 'TRANSFORM', 'MAP', 'LIST_TRANSFORM',
    'LIST_FILTER', 'LIST_APPLY', 'LIST_REDUCE', 'LIST_SORT',
    'LIST_DISTINCT', 'LIST_UNIQUE', 'LIST_HAS', 'LIST_INDEXOF',
    'LIST_SLICE', 'LIST_EXTRACT', 'LIST_VALUE', 'LIST_AGGREGATE'
  ],
  mssql: [
    'ISNULL', 'COALESCE', 'NULLIF', 'IIF', 'CHOOSE',
    'CONCAT', 'CONCAT_WS', 'FORMAT', 'STRING_AGG',
    'STRING_ESCAPE', 'QUOTENAME', 'PARSENAME', 'SUSER_NAME',
    'USER_NAME', 'CURRENT_USER', 'SYSTEM_USER', 'SESSION_USER',
    'ORIGINAL_LOGIN', 'HOST_NAME', 'APP_NAME', 'DB_NAME',
    'OBJECT_NAME', 'OBJECT_ID', 'SCHEMA_NAME', 'TYPE_NAME',
    'COL_NAME', 'INDEX_COL', 'INDEXPROPERTY', 'COLUMNPROPERTY',
    'DATABASEPROPERTYEX', 'OBJECTPROPERTY', 'OBJECTPROPERTYEX',
    'INDEXPROPERTY', 'FILEPROPERTY', 'FILEGROUPPROPERTY',
    'FULLTEXTCATALOGPROPERTY', 'FULLTEXTSERVICEPROPERTY'
  ],
  oracle: [
    'NVL', 'NVL2', 'NULLIF', 'COALESCE', 'DECODE',
    'TO_CHAR', 'TO_NUMBER', 'TO_DATE', 'TO_TIMESTAMP',
    'TO_CLOB', 'TO_BLOB', 'TO_NCLOB', 'CAST', 'CONVERT',
    'TRANSLATE', 'REPLACE', 'SUBSTR', 'INSTR', 'LENGTH',
    'LPAD', 'RPAD', 'TRIM', 'LTRIM', 'RTRIM', 'UPPER',
    'LOWER', 'INITCAP', 'SOUNDEX', 'ASCII', 'CHR',
    'CONCAT', 'REGEXP_REPLACE', 'REGEXP_SUBSTR', 'REGEXP_INSTR',
    'REGEXP_COUNT', 'REGEXP_LIKE', 'LISTAGG', 'XMLAGG',
    'WM_CONCAT', 'SYS_CONNECT_BY_PATH', 'LAG', 'LEAD',
    'FIRST_VALUE', 'LAST_VALUE', 'NTH_VALUE', 'RANK',
    'DENSE_RANK', 'ROW_NUMBER', 'NTILE', 'CUME_DIST',
    'PERCENT_RANK', 'PERCENTILE_CONT', 'PERCENTILE_DISC'
  ],
  snowflake: [
    'ARRAY_AGG', 'ARRAY_APPEND', 'ARRAY_CAT', 'ARRAY_COMPACT',
    'ARRAY_CONSTRUCT', 'ARRAY_CONTAINS', 'ARRAY_INSERT',
    'ARRAY_INTERSECTION', 'ARRAY_POSITION', 'ARRAY_PREPEND',
    'ARRAY_SIZE', 'ARRAY_SLICE', 'ARRAY_TO_STRING',
    'ARRAY_UNION_AGG', 'ARRAY_UNIQUE_AGG', 'OBJECT_AGG',
    'OBJECT_CONSTRUCT', 'OBJECT_DELETE', 'OBJECT_INSERT',
    'OBJECT_KEYS', 'OBJECT_PICK', 'GET_PATH', 'CHECK_JSON',
    'CHECK_XML', 'PARSE_JSON', 'PARSE_XML', 'XMLGET',
    'TO_JSON', 'TO_VARCHAR', 'TO_CHAR', 'TO_NUMBER',
    'TO_DECIMAL', 'TO_DOUBLE', 'TO_BOOLEAN', 'TO_DATE',
    'TO_TIME', 'TO_TIMESTAMP', 'TO_TIMESTAMP_LTZ', 'TO_TIMESTAMP_NTZ',
    'TO_TIMESTAMP_TZ', 'TRY_TO_DATE', 'TRY_TO_TIME', 'TRY_TO_TIMESTAMP',
    'DATE_PART', 'DATE_TRUNC', 'DATEDIFF', 'TIMESTAMPDIFF',
    'LAST_DAY', 'NEXT_DAY', 'PREVIOUS_DAY', 'TIMEADD',
    'TIMESTAMPADD', 'DATEADD', 'TIME_SLICE', 'TIMESTAMP_SLICE'
  ],
  bigquery: [
    'ARRAY_AGG', 'ARRAY_CONCAT', 'ARRAY_CONCAT_AGG',
    'ARRAY_LENGTH', 'ARRAY_TO_STRING', 'GENERATE_ARRAY',
    'GENERATE_DATE_ARRAY', 'GENERATE_TIMESTAMP_ARRAY',
    'ARRAY_REVERSE', 'OFFSET', 'ORDINAL', 'SAFE_OFFSET',
    'SAFE_ORDINAL', 'JSON_EXTRACT', 'JSON_EXTRACT_SCALAR',
    'JSON_QUERY', 'JSON_VALUE', 'JSON_QUERY_ARRAY',
    'JSON_VALUE_ARRAY', 'PARSE_JSON', 'TO_JSON',
    'TO_JSON_STRING', 'JSON_TYPE', 'SAFE_CAST',
    'SAFE_CONVERT_BYTES_TO_STRING', 'FORMAT_DATE',
    'PARSE_DATE', 'FORMAT_TIME', 'PARSE_TIME',
    'FORMAT_DATETIME', 'PARSE_DATETIME', 'FORMAT_TIMESTAMP',
    'PARSE_TIMESTAMP', 'TIMESTAMP_SECONDS', 'TIMESTAMP_MILLIS',
    'TIMESTAMP_MICROS', 'UNIX_SECONDS', 'UNIX_MILLIS',
    'UNIX_MICROS', 'FARM_FINGERPRINT', 'MD5', 'SHA1',
    'SHA256', 'SHA512', 'REGEXP_CONTAINS', 'REGEXP_EXTRACT',
    'REGEXP_EXTRACT_ALL', 'REGEXP_REPLACE'
  ],
  redshift: [
    'ARRAY_AGG', 'LISTAGG', 'MEDIAN', 'PERCENTILE_CONT',
    'PERCENTILE_DISC', 'APPROXIMATE PERCENTILE_DISC',
    'STDDEV', 'STDDEV_SAMP', 'STDDEV_POP', 'VAR_SAMP',
    'VAR_POP', 'VARIANCE', 'COVAR_POP', 'COVAR_SAMP',
    'CORR', 'REGR_AVGX', 'REGR_AVGY', 'REGR_COUNT',
    'REGR_INTERCEPT', 'REGR_R2', 'REGR_SLOPE', 'REGR_SXX',
    'REGR_SXY', 'REGR_SYY', 'BIT_AND', 'BIT_OR', 'BOOL_AND',
    'BOOL_OR', 'EVERY', 'ANY_VALUE', 'FIRST_VALUE', 'LAST_VALUE',
    'NTH_VALUE', 'LAG', 'LEAD', 'RANK', 'DENSE_RANK',
    'ROW_NUMBER', 'NTILE', 'CUME_DIST', 'PERCENT_RANK',
    'DATE_PART', 'DATE_TRUNC', 'DATEDIFF', 'DATEADD',
    'ADD_MONTHS', 'LAST_DAY', 'NEXT_DAY', 'PREVIOUS_DAY',
    'TIMEOFDAY', 'GETDATE', 'SYSDATE', 'CURRENT_DATE',
    'CURRENT_TIME', 'CURRENT_TIMESTAMP', 'LOCALTIME',
    'LOCALTIMESTAMP', 'TIMEZONE', 'TIMEZONE_HOUR', 'TIMEZONE_MINUTE'
  ]
}

// 已注册的高亮提供者
const registeredHighlighters = new Map<SqlDialect, monaco.IDisposable>()

/**
 * 注册方言高亮规则
 */
export function registerDialectHighlight(dialect: SqlDialect): monaco.IDisposable {
  // 如果已经注册，先注销
  if (registeredHighlighters.has(dialect)) {
    registeredHighlighters.get(dialect)!.dispose()
  }

  const keywords = DIALECT_KEYWORDS[dialect] || []
  const functions = DIALECT_FUNCTIONS[dialect] || []

  // 注册方言关键字
  const keywordDisposable = monaco.languages.setMonarchTokensProvider('sql', {
    keywords: [
      'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE',
      'CREATE', 'DROP', 'ALTER', 'TABLE', 'INDEX', 'VIEW',
      'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'ON',
      'GROUP', 'BY', 'ORDER', 'HAVING', 'LIMIT', 'OFFSET',
      'AND', 'OR', 'NOT', 'NULL', 'IS', 'IN', 'EXISTS',
      'COUNT', 'SUM', 'AVG', 'MAX', 'MIN', 'AS', 'DISTINCT',
      ...keywords
    ],
    operators: [
      '=', '>', '<', '!', '~', '?', ':', '==', '<=', '>=',
      '!=', '<>', '=>', '&&', '||', '++', '--', '+', '-',
      '*', '/', '&', '|', '^', '%', '<<', '>>', '>>>',
      '&&&', '|||', '??', '???'
    ],
    builtinFunctions: functions,
    tokenizer: {
      root: [
        // 注释
        [/\/\*/, 'comment', '@comment'],
        [/--.*$/, 'comment'],
        
        // 字符串
        [/'/, 'string', '@string'],
        
        // 数字
        [/\d*\.\d+([eE][+-]?\d+)?/, 'number'],
        [/\d+([eE][+-]?\d+)?/, 'number'],
        
        // 标识符
        [/[a-zA-Z_]\w*/, {
          cases: {
            '@keywords': 'keyword',
            '@builtinFunctions': 'function',
            '@default': 'identifier'
          }
        }],
        
        // 操作符
        [/[@$][a-zA-Z_]\w*/, 'variable'],
        [/[{}()[\]]/, '@brackets'],
        [/./, 'delimiter']
      ],
      comment: [
        [/[^/*]+/, 'comment'],
        [/\*\//, 'comment', '@pop'],
        [/[/*]/, 'comment']
      ],
      string: [
        [/[^']+/, 'string'],
        [/''/, 'string.escape'],
        [/'/, 'string', '@pop']
      ]
    }
  })

  registeredHighlighters.set(dialect, keywordDisposable)
  return keywordDisposable
}

/**
 * 注销方言高亮规则
 */
export function unregisterDialectHighlight(dialect: SqlDialect): void {
  if (registeredHighlighters.has(dialect)) {
    registeredHighlighters.get(dialect)!.dispose()
    registeredHighlighters.delete(dialect)
  }
}

/**
 * 获取方言关键字列表
 */
export function getDialectKeywords(dialect: SqlDialect): string[] {
  return DIALECT_KEYWORDS[dialect] || []
}

/**
 * 获取方言函数列表
 */
export function getDialectFunctions(dialect: SqlDialect): string[] {
  return DIALECT_FUNCTIONS[dialect] || []
}

/**
 * 清理所有高亮规则
 */
export function cleanupAllHighlights(): void {
  registeredHighlighters.forEach((disposable) => {
    disposable.dispose()
  })
  registeredHighlighters.clear()
}
