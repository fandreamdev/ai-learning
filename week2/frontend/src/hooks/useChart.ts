import { useState, useCallback } from 'react';
import { apiClient } from '@/api/client';

export type ChartType = 'line' | 'bar' | 'pie' | 'scatter' | 'radar' | 'funnel' | 'gauge';

export interface ChartRecommendResult {
  recommended_types: ChartType[];
  reasons: string[];
}

export interface ChartGenerateResult {
  chart_type: ChartType;
  config: Record<string, unknown>;
}

export interface ChartData {
  columns: Array<{ name: string; data_type: string }>;
  rows: unknown[][];
}

export function useChart() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const recommend = useCallback(async (columns: ChartData['columns'], rowCount: number) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{ data: ChartRecommendResult }>('/charts/recommend', {
        columns,
        row_count: rowCount,
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
    data: ChartData,
    options?: {
      title?: string;
      xAxis?: string;
      yAxis?: string;
      seriesName?: string;
    }
  ) => {
    setLoading(true);
    setError(null);

    try {
      const xData = data.rows.map(row => String(row[0]));
      const seriesData = data.rows.map(row => {
        const val = row[1];
        if (typeof val === 'number') return val;
        if (typeof val === 'string') return parseFloat(val) || 0;
        return 0;
      });

      const response = await apiClient.post<{ data: ChartGenerateResult }>('/charts/generate', {
        chart_type: chartType,
        title: options?.title || '数据图表',
        x_data: xData,
        series: seriesData,
        series_name: options?.seriesName || '数值',
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
    format: 'png' | 'jpg' | 'svg' | 'pdf' = 'png'
  ) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{
        data: { format: string; url: string; filename: string };
      }>('/charts/export', {
        chart_config: chartConfig,
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
