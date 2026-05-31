import { useState, useCallback } from 'react';
import { apiClient } from '@/api/client';

export interface SqlExecuteResult {
  query_id: string;
  columns: Array<{ name: string; data_type: string }>;
  rows: unknown[][];
  row_count: number;
  total: number;
  page: number;
  page_size: number;
  duration_ms: number;
}

export interface SqlExecuteRequest {
  connection_id: string;
  sql: string;
  timeout?: number;
  page?: number;
  page_size?: number;
}

export function useSqlExecute() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<SqlExecuteResult | null>(null);

  const execute = useCallback(async (request: SqlExecuteRequest) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{ data: SqlExecuteResult }>('/sql/execute', request);
      setResult(response.data.data);
      return response.data.data;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'SQL 执行失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const format = useCallback(async (sql: string, dialect?: string) => {
    try {
      const response = await apiClient.post<{ data: { formatted_sql: string } }>('/sql/format', {
        sql,
        dialect,
      });
      return response.data.data.formatted_sql;
    } catch (err) {
      console.error('SQL 格式化失败:', err);
      return sql;
    }
  }, []);

  const getHistory = useCallback(async (page = 1, pageSize = 50) => {
    try {
      const response = await apiClient.get<{
        data: { items: unknown[]; total: number; page: number; page_size: number };
      }>('/sql/history', { params: { page, page_size: pageSize } });
      return response.data.data;
    } catch (err) {
      console.error('获取查询历史失败:', err);
      return { items: [], total: 0, page: 1, page_size: pageSize };
    }
  }, []);

  const explain = useCallback(async (request: SqlExecuteRequest) => {
    try {
      const response = await apiClient.post<{
        data: {
          plan_type: string;
          estimated_cost: number | null;
          estimated_rows: number | null;
          actual_rows: number | null;
          details: Record<string, unknown>;
        };
      }>('/sql/explain', request);
      return response.data.data;
    } catch (err) {
      console.error('获取执行计划失败:', err);
      return null;
    }
  }, []);

  const reset = useCallback(() => {
    setLoading(false);
    setError(null);
    setResult(null);
  }, []);

  return {
    loading,
    error,
    result,
    execute,
    format,
    getHistory,
    explain,
    reset,
  };
}
