import { useEffect, useRef, useState, useCallback } from 'react';
import * as echarts from 'echarts';
import type { ECharts, EChartsOption } from 'echarts';

// 图表类型
export type ChartType = 'line' | 'bar' | 'pie' | 'scatter' | 'radar' | 'funnel' | 'gauge';

export interface ChartData {
  columns: Array<{ name: string; data_type: string }>;
  rows: Array<Record<string, unknown>>;
}

export interface ChartConfig {
  type: ChartType;
  title?: string;
  xAxis?: string;
  yAxis?: string | string[];
  series?: string[];
  radius?: string | [string, string];
  center?: [string, string];
}

interface ChartRendererProps {
  data: ChartData | null;
  config: ChartConfig;
  loading?: boolean;
  onChartClick?: (params: unknown) => void;
  height?: number | string;
  className?: string;
}

const CHART_COLORS = [
  '#5470c6',
  '#91cc75',
  '#fac858',
  '#ee6666',
  '#73c0de',
  '#3ba272',
  '#fc8452',
  '#9a60b4',
  '#ea7ccc',
];

export function ChartRenderer({
  data,
  config,
  loading = false,
  onChartClick,
  height = 400,
  className = '',
}: ChartRendererProps) {
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstance = useRef<ECharts | null>(null);
  const [isReady, setIsReady] = useState(false);

  // 初始化图表
  useEffect(() => {
    if (!chartRef.current) return;

    const chart = echarts.init(chartRef.current, undefined, {
      renderer: 'canvas',
    });

    chartInstance.current = chart;
    setIsReady(true);

    // 绑定点击事件
    if (onChartClick) {
      chart.on('click', onChartClick);
    }

    // 响应窗口大小变化
    const handleResize = () => {
      chart.resize();
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.off('click', onChartClick);
      chart.dispose();
    };
  }, [onChartClick]);

  // 更新图表数据
  useEffect(() => {
    if (!chartInstance.current || !isReady || !data) return;

    const option = buildChartOption(data, config);
    chartInstance.current.setOption(option, true);
  }, [data, config, isReady]);

  // 构建图表配置
  const buildChartOption = useCallback((chartData: ChartData, cfg: ChartConfig): EChartsOption => {
    const baseOption: EChartsOption = {
      backgroundColor: 'transparent',
      color: CHART_COLORS,
      title: cfg.title
        ? {
            text: cfg.title,
            left: 'center',
            textStyle: { fontSize: 16, fontWeight: 500 },
          }
        : undefined,
      tooltip: {
        trigger: cfg.type === 'pie' ? 'item' : 'axis',
        backgroundColor: 'rgba(255, 255, 255, 0.95)',
        borderColor: '#e5e7eb',
        textStyle: { color: '#374151' },
      },
      legend:
        cfg.type === 'pie'
          ? {
              bottom: 10,
              left: 'center',
            }
          : undefined,
      grid:
        cfg.type !== 'pie' && cfg.type !== 'gauge'
          ? {
              left: '3%',
              right: '4%',
              bottom: '3%',
              containLabel: true,
            }
          : undefined,
    };

    switch (cfg.type) {
      case 'line':
        return buildLineChart(baseOption, chartData, cfg);
      case 'bar':
        return buildBarChart(baseOption, chartData, cfg);
      case 'pie':
        return buildPieChart(baseOption, chartData, cfg);
      case 'scatter':
        return buildScatterChart(baseOption, chartData, cfg);
      case 'radar':
        return buildRadarChart(baseOption, chartData, cfg);
      case 'gauge':
        return buildGaugeChart(baseOption, chartData, cfg);
      default:
        return buildLineChart(baseOption, chartData, cfg);
    }
  }, []);

  // 渲染加载状态
  if (loading) {
    return (
      <div
        className={`flex items-center justify-center bg-gray-50 rounded-lg ${className}`}
        style={{ height }}
      >
        <div className="flex flex-col items-center gap-2">
          <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-500">加载图表...</span>
        </div>
      </div>
    );
  }

  // 渲染空状态
  if (!data || data.rows.length === 0) {
    return (
      <div
        className={`flex items-center justify-center bg-gray-50 rounded-lg ${className}`}
        style={{ height }}
      >
        <div className="text-center text-gray-400">
          <svg
            className="w-12 h-12 mx-auto mb-2"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            />
          </svg>
          <p className="text-sm">暂无数据</p>
        </div>
      </div>
    );
  }

  return (
    <div
      ref={chartRef}
      className={className}
      style={{ height }}
    />
  );
}

// 折线图
function buildLineChart(
  base: EChartsOption,
  data: ChartData,
  cfg: ChartConfig
): EChartsOption {
  const xKey = cfg.xAxis || data.columns[0]?.name;
  const yKeys = Array.isArray(cfg.yAxis) ? cfg.yAxis : cfg.yAxis ? [cfg.yAxis] : data.columns.slice(1).map((c) => c.name);

  return {
    ...base,
    xAxis: {
      type: 'category',
      data: data.rows.map((r) => String(r[xKey] ?? '')),
      boundaryGap: false,
    },
    yAxis: {
      type: 'value',
    },
    series: yKeys.map((yKey) => ({
      name: yKey,
      type: 'line',
      data: data.rows.map((r) => toChartValue(r[yKey])),
      smooth: true,
      emphasis: { focus: 'series' },
    })),
  };
}

// 柱状图
function buildBarChart(
  base: EChartsOption,
  data: ChartData,
  cfg: ChartConfig
): EChartsOption {
  const xKey = cfg.xAxis || data.columns[0]?.name;
  const yKeys = Array.isArray(cfg.yAxis) ? cfg.yAxis : cfg.yAxis ? [cfg.yAxis] : data.columns.slice(1).map((c) => c.name);

  return {
    ...base,
    xAxis: {
      type: 'category',
      data: data.rows.map((r) => String(r[xKey] ?? '')),
    },
    yAxis: {
      type: 'value',
    },
    series: yKeys.map((yKey) => ({
      name: yKey,
      type: 'bar',
      data: data.rows.map((r) => toChartValue(r[yKey])),
      emphasis: { focus: 'series' },
    })),
  };
}

// 饼图
function buildPieChart(
  base: EChartsOption,
  data: ChartData,
  cfg: ChartConfig
): EChartsOption {
  const nameKey = data.columns[0]?.name;
  const valueKey = data.columns[1]?.name;

  return {
    ...base,
    series: [
      {
        name: valueKey,
        type: 'pie',
        radius: cfg.radius || '50%',
        center: cfg.center || ['50%', '50%'],
        data: data.rows.map((r) => ({
          name: String(r[nameKey] ?? ''),
          value: Number(r[valueKey]) || 0,
        })),
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
        label: {
          formatter: '{b}: {d}%',
        },
      },
    ],
  };
}

// 散点图
function buildScatterChart(
  base: EChartsOption,
  data: ChartData,
  cfg: ChartConfig
): EChartsOption {
  const xKey = cfg.xAxis || data.columns[0]?.name;
  const yKey = Array.isArray(cfg.yAxis) ? cfg.yAxis[0] : cfg.yAxis || data.columns[1]?.name;

  return {
    ...base,
    xAxis: {
      type: 'value',
      name: xKey,
    },
    yAxis: {
      type: 'value',
      name: yKey,
    },
    series: [
      {
        type: 'scatter',
        symbolSize: 10,
        data: data.rows.map((r) => [Number(r[xKey]) || 0, Number(r[yKey]) || 0]),
      },
    ],
  };
}

// 雷达图
function buildRadarChart(
  base: EChartsOption,
  data: ChartData,
  _cfg: ChartConfig
): EChartsOption {
  const indicatorKey = data.columns[0]?.name;
  const valueKeys = data.columns.slice(1).map((c) => c.name);

  const maxValues = valueKeys.map((key) =>
    Math.max(...data.rows.map((r) => Math.abs(Number(r[key]) || 0)))
  );

  return {
    ...base,
    radar: {
      indicator: data.rows.map((r, i) => ({
        name: String(r[indicatorKey] ?? `指标${i + 1}`),
        max: maxValues[data.rows.indexOf(r)] * 1.2 || 100,
      })),
      radius: '65%',
    },
    series: [
      {
        type: 'radar',
        data: [
          {
            value: valueKeys.map((key) => Number(data.rows[0]?.[key]) || 0),
            name: '实际值',
          },
        ],
        emphasis: { focus: 'series' },
      },
    ],
  };
}

// 仪表盘
function buildGaugeChart(
  base: EChartsOption,
  data: ChartData,
  cfg: ChartConfig
): EChartsOption {
  const valueKey = data.columns[1]?.name || data.columns[0]?.name;
  const value = Number(data.rows[0]?.[valueKey]) || 0;

  return {
    ...base,
    series: [
      {
        type: 'gauge',
        startAngle: 180,
        endAngle: 0,
        center: ['50%', '75%'],
        radius: '90%',
        min: 0,
        max: 100,
        splitNumber: 8,
        axisLine: {
          lineStyle: {
            width: 6,
            color: [
              [0.3, '#fd666d'],
              [0.7, '#37a2da'],
              [1, '#67e0e3'],
            ],
          },
        },
        pointer: {
          icon: 'path://M12.8,0.7l12,40.1H0.7L12.8,0.7z',
          length: '12%',
          width: 20,
          offsetCenter: [0, '-60%'],
          itemStyle: { color: 'auto' },
        },
        axisTick: { length: 12, lineStyle: { color: 'auto', width: 2 } },
        splitLine: { length: 20, lineStyle: { color: 'auto', width: 5 } },
        axisLabel: {
          color: '#464646',
          fontSize: 12,
          distance: -60,
          formatter: '{value}',
        },
        title: {
          offsetCenter: [0, '-10%'],
          fontSize: 16,
        },
        detail: {
          fontSize: 30,
          offsetCenter: [0, '-35%'],
          valueAnimation: true,
          formatter: (val: number) => Math.round(val) + '%',
          color: 'auto',
        },
        data: [{ value, name: cfg.title || '完成率' }],
      },
    ],
  };
}

function toChartValue(value: unknown): string | number | null {
  if (typeof value === 'number' || typeof value === 'string') return value;
  if (typeof value === 'boolean') return Number(value);
  if (value == null) return null;
  const numeric = Number(value);
  return Number.isFinite(numeric) ? numeric : String(value);
}

export default ChartRenderer;
