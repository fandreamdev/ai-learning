export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  token_type: string
  user: User
}

export interface User {
  id: string
  username: string
  email: string
  role: UserRole
  is_active: boolean
  created_at: string
}

export type UserPublic = User

export type UserRole = 'admin' | 'analyst' | 'developer' | 'business'

export interface DatabaseConnection {
  id: string
  name: string
  db_type: DatabaseType
  host: string
  port: number
  database_name: string
  username: string
  is_default: boolean
  status?: boolean
  created_at: string
}

export type Connection = DatabaseConnection

export interface CreateConnectionRequest {
  name: string
  db_type: DatabaseType
  host: string
  port: number
  database_name: string
  username: string
  password: string
  is_default?: boolean
}

export type DatabaseType = 'mysql' | 'postgresql' | 'clickhouse' | 'sqlite'

export interface SqlExecuteRequest {
  connection_id: string
  sql: string
  timeout?: number
  explain?: boolean
}

export interface SqlExecuteResponse {
  query_id: string
  columns: ColumnMetadata[]
  rows: unknown[][]
  row_count: number
  duration_ms: number
  execution_plan?: ExecutionPlan
}

export interface ColumnMetadata {
  name: string
  data_type: string
  ordinal: number
}

export interface ExecutionPlan {
  plan_type: string
  estimated_cost?: number
  estimated_rows?: number
  actual_rows?: number
  details: unknown
}

export interface Conversation {
  id: string
  title: string
  created_at: string
  updated_at: string
}

export interface Message {
  id: string
  conversation_id: string
  role: 'user' | 'assistant'
  content: string
  generated_sql?: string
  sql_explanation?: string
  execution_result?: unknown
  chart_config?: unknown
  created_at: string
}

export interface NlConvertRequest {
  connection_id: string
  question: string
  conversation_id?: string
}

export interface NlConvertResponse {
  sql: string
  explanation: string
  confidence: number
  estimated_rows?: number
  referenced_tables: string[]
}

export interface ChartConfig {
  chart_type: string
  title: string
  x_field?: string
  y_field?: string
  series_field?: string
  value_fields: string[]
  echarts_config: unknown
}

export interface ApiResponse<T> {
  code: number
  message?: string
  data?: T
}
