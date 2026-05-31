import { useState, useCallback } from 'react';
import { apiClient } from '@/api/client';

export interface NlConvertResult {
  sql: string;
  explanation: string;
  confidence: number;
  estimated_rows: number | null;
  referenced_tables: string[];
}

export interface NlConvertRequest {
  question: string;
  connection_id?: string;
}

export function useNlConvert() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<NlConvertResult | null>(null);

  const convert = useCallback(async (request: NlConvertRequest) => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiClient.post<{ data: NlConvertResult }>('/nl/convert', request);
      setResult(response.data.data);
      return response.data.data;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'NL 转 SQL 失败';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
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
    convert,
    reset,
  };
}
