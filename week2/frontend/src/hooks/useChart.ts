import { useState, useCallback } from 'react';
import { apiClient } from '@/api/client';

export type ChartType = 'line' | 'bar' | 'pie' | 'scatter' | 'radar' | 'funnel' | 'gauge';

export interface ChartRecommendResult {
  recommended: ChartType;
  recommended_types: ChartType[];
  reasons: string[];
  chart_config?: Record<string, unknown>;
}

export interface ChartGenerateResult {
  chart_type: ChartType;
  config: Record<string, unknown>;
}

export interface ChartData {
  columns: Array<{ name: string; data_type: string; ordinal?: number }>;
  rows: unknown[][];
}

export function useChart() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const recommend = useCallback(async (data: ChartData) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{ data: ChartRecommendResult }>('/charts/recommend', {
        columns: data.columns.map((column, index) => ({
          ...column,
          ordinal: column.ordinal ?? index,
        })),
        rows: data.rows,
      });
      return response.data.data;
    } catch (err) {
      const message = err instanceof Error ? err.message : '图表推荐失败';
      setError(message);
      return {
        recommended_types: ['bar', 'line', 'pie'] as ChartType[],
        reasons: ['数据适合用柱状图展示', '时间序列数据适合用折线图', '比例数据适合用饼图'],
      };
    } finally {
      setLoading(false);
    }
  }, []);

  const generate = useCallback(async (
    chartType: ChartType,
    data: ChartData
  ) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{ data: ChartGenerateResult }>('/charts/generate', {
        columns: data.columns.map((column, index) => ({
          ...column,
          ordinal: column.ordinal ?? index,
        })),
        rows: data.rows,
        chart_type: chartType,
      });
      return response.data.data;
    } catch (err) {
      const message = err instanceof Error ? err.message : '图表生成失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const exportChart = useCallback(async (
    chartConfig: Record<string, unknown>,
    format: 'json' | 'svg' | 'png' = 'svg'
  ) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{
        data: { format: string; url: string; filename: string };
      }>('/charts/export', {
        config: chartConfig,
        format,
      });
      return response.data.data;
    } catch (err) {
      const message = err instanceof Error ? err.message : '图表导出失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const reset = useCallback(() => {
    setLoading(false);
    setError(null);
  }, []);

  return {
    loading,
    error,
    recommend,
    generate,
    exportChart,
    reset,
  };
}
