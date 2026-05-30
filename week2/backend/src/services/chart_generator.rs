//! 图表生成服务
//!
//! 提供智能图表推荐和数据可视化配置生成

use crate::error::AppResult;
use crate::models::query::ColumnMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    /// 折线图
    Line,
    /// 柱状图
    Bar,
    /// 饼图
    Pie,
    /// 环形图
    Doughnut,
    /// 散点图
    Scatter,
    /// 地图
    Map,
    /// 漏斗图
    Funnel,
    /// 雷达图
    Radar,
    /// 仪表盘
    Gauge,
    /// 表格
    Table,
}

impl ChartType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Bar => "bar",
            Self::Pie => "pie",
            Self::Doughnut => "doughnut",
            Self::Scatter => "scatter",
            Self::Map => "map",
            Self::Funnel => "funnel",
            Self::Radar => "radar",
            Self::Gauge => "gauge",
            Self::Table => "table",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Line => "折线图",
            Self::Bar => "柱状图",
            Self::Pie => "饼图",
            Self::Doughnut => "环形图",
            Self::Scatter => "散点图",
            Self::Map => "地图",
            Self::Funnel => "漏斗图",
            Self::Radar => "雷达图",
            Self::Gauge => "仪表盘",
            Self::Table => "数据表格",
        }
    }
}

/// 图表配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// 图表类型
    pub chart_type: String,

    /// 标题
    pub title: String,

    /// X 轴字段
    pub x_field: Option<String>,

    /// Y 轴字段
    pub y_field: Option<String>,

    /// 系列字段（用于多系列图表）
    pub series_field: Option<String>,

    /// 数值字段
    pub value_fields: Vec<String>,

    /// ECharts 配置
    pub echarts_config: serde_json::Value,
}

/// 图表推荐结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartRecommendation {
    /// 推荐图表类型
    pub recommended: ChartType,

    /// 备选图表类型
    pub alternatives: Vec<ChartType>,

    /// 推荐理由
    pub reasons: Vec<String>,

    /// 生成图表配置
    pub chart_config: ChartConfig,
}

/// 图表生成服务
#[derive(Clone)]
pub struct ChartGenerator {
    recommendations: HashMap<String, ChartRecommendation>,
}

impl ChartGenerator {
    pub fn new() -> Self {
        Self {
            recommendations: HashMap::new(),
        }
    }

    /// 推荐图表类型
    pub fn recommend(&self, columns: &[ColumnMetadata], rows: &[Vec<serde_json::Value>]) -> AppResult<ChartRecommendation> {
        // 分析数据特征
        let analysis = self.analyze_data(columns, rows);

        // 根据特征推荐图表
        let (recommended, reasons) = self.select_chart(&analysis);

        // 生成配置
        let chart_config = self.generate_config(&recommended, columns, rows, &analysis)?;

        Ok(ChartRecommendation {
            recommended,
            alternatives: self.get_alternatives(&analysis),
            reasons,
            chart_config,
        })
    }

    /// 分析数据特征
    fn analyze_data(&self, columns: &[ColumnMetadata], rows: &[Vec<serde_json::Value>]) -> DataAnalysis {
        let col_count = columns.len();
        let row_count = rows.len();

        // 检测数据类型
        let mut numeric_cols = Vec::new();
        let mut categorical_cols = Vec::new();
        let mut date_cols = Vec::new();

        for (i, col) in columns.iter().enumerate() {
            let data_type = col.data_type.to_lowercase();

            if data_type.contains("int") || data_type.contains("decimal") || data_type.contains("float") || data_type.contains("double") {
                numeric_cols.push(i);
            } else if data_type.contains("date") || data_type.contains("time") || data_type.contains("timestamp") {
                date_cols.push(i);
            } else {
                // 检查实际数据
                if let Some(first_row) = rows.first() {
                    if let Some(value) = first_row.get(i) {
                        if value.is_string() {
                            // 估算类别数量
                            let unique_count = rows.iter()
                                .filter_map(|r| r.get(i).and_then(|v| v.as_str()))
                                .collect::<std::collections::HashSet<_>>()
                                .len();

                            if unique_count <= 20 {
                                categorical_cols.push(i);
                            }
                        }
                    }
                }
            }
        }

        DataAnalysis {
            row_count,
            col_count,
            numeric_cols,
            categorical_cols,
            date_cols,
        }
    }

    /// 根据分析选择图表
    fn select_chart(&self, analysis: &DataAnalysis) -> (ChartType, Vec<String>) {
        let mut reasons = Vec::new();
        let chart_type;

        // 时间序列数据 -> 折线图
        if !analysis.date_cols.is_empty() && !analysis.numeric_cols.is_empty() {
            chart_type = ChartType::Line;
            reasons.push("检测到时间序列数据，适合使用折线图展示趋势".to_string());
        }
        // 少量分类 + 数值 -> 柱状图或饼图
        else if !analysis.categorical_cols.is_empty() && !analysis.numeric_cols.is_empty() {
            let cat_col = analysis.categorical_cols[0];
            let unique_count = 10; // 简化处理

            if unique_count <= 6 {
                chart_type = ChartType::Pie;
                reasons.push("分类数量较少，适合使用饼图展示占比".to_string());
            } else {
                chart_type = ChartType::Bar;
                reasons.push("适合使用柱状图进行分类对比".to_string());
            }
        }
        // 多列数值数据 -> 散点图或雷达图
        else if analysis.numeric_cols.len() >= 2 {
            chart_type = ChartType::Scatter;
            reasons.push("多维数值数据，适合使用散点图展示相关性".to_string());
        }
        // 默认表格
        else {
            chart_type = ChartType::Table;
            reasons.push("数据格式适合表格展示".to_string());
        }

        // 添加数据量信息
        reasons.push(format!("数据量: {} 行, {} 列", analysis.row_count, analysis.col_count));

        (chart_type, reasons)
    }

    /// 获取备选图表类型
    fn get_alternatives(&self, analysis: &DataAnalysis) -> Vec<ChartType> {
        let mut alternatives = vec![];

        if !analysis.date_cols.is_empty() {
            alternatives.push(ChartType::Bar);
        } else if !analysis.categorical_cols.is_empty() {
            alternatives.push(ChartType::Line);
            if analysis.numeric_cols.len() == 1 {
                alternatives.push(ChartType::Pie);
            }
        }

        alternatives.push(ChartType::Table);

        alternatives.truncate(3);
        alternatives
    }

    /// 生成图表配置
    fn generate_config(
        &self,
        chart_type: &ChartType,
        columns: &[ColumnMetadata],
        rows: &[Vec<serde_json::Value>],
        _analysis: &DataAnalysis,
    ) -> AppResult<ChartConfig> {
        // 确定 X 轴和 Y 轴字段
        let x_field = columns.first().map(|c| c.name.clone());
        let y_field = columns.get(1).map(|c| c.name.clone());
        let value_fields: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

        // 生成 ECharts 配置
        let echarts_config = self.build_echarts_config(chart_type, columns, rows)?;

        Ok(ChartConfig {
            chart_type: chart_type.as_str().to_string(),
            title: "数据可视化".to_string(),
            x_field,
            y_field,
            series_field: None,
            value_fields,
            echarts_config,
        })
    }

    /// 构建 ECharts 配置
    fn build_echarts_config(
        &self,
        chart_type: &ChartType,
        columns: &[ColumnMetadata],
        rows: &[Vec<serde_json::Value>],
    ) -> AppResult<serde_json::Value> {
        let mut config = serde_json::json!({
            "responsive": true,
            "maintainAspectRatio": true,
        });

        // 根据图表类型生成配置
        match chart_type {
            ChartType::Line | ChartType::Bar => {
                config["xAxis"] = serde_json::json!({
                    "type": "category",
                    "data": rows.iter().filter_map(|r| r.first().and_then(|v| v.as_str())).collect::<Vec<_>>()
                });
                config["yAxis"] = serde_json::json!({
                    "type": "value"
                });
                config["series"] = vec![serde_json::json!({
                    "type": chart_type.as_str(),
                    "data": rows.iter().filter_map(|r| r.get(1).map(|v| {
                        if let Some(n) = v.as_i64() {
                            serde_json::Value::Number(n.into())
                        } else if let Some(n) = v.as_f64() {
                            serde_json::Number::from_f64(n).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)
                        } else {
                            serde_json::Value::Null
                        }
                    })).collect::<Vec<_>>()
                })];
            }
            ChartType::Pie | ChartType::Doughnut => {
                config["series"] = vec![serde_json::json!({
                    "type": chart_type.as_str(),
                    "radius": if chart_type == &ChartType::Doughnut { "55%" } else { "60%" },
                    "data": rows.iter().map(|r| {
                        let name = r.first().and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let value = r.get(1).map(|v| {
                            if let Some(n) = v.as_i64() {
                                n
                            } else if let Some(n) = v.as_f64() {
                                n as i64
                            } else {
                                0
                            }
                        }).unwrap_or(0);
                        serde_json::json!({ "name": name, "value": value })
                    }).collect::<Vec<_>>()
                })];
            }
            _ => {}
        }

        Ok(config)
    }

    /// 转换图表类型
    pub fn switch_chart_type(
        &self,
        columns: &[ColumnMetadata],
        rows: &[Vec<serde_json::Value>],
        new_type: &str,
    ) -> AppResult<ChartConfig> {
        let chart_type = match new_type {
            "line" => ChartType::Line,
            "bar" => ChartType::Bar,
            "pie" => ChartType::Pie,
            "doughnut" => ChartType::Doughnut,
            "scatter" => ChartType::Scatter,
            "table" => ChartType::Table,
            _ => return Err(crate::error::AppError::validation("不支持的图表类型".to_string())),
        };

        let analysis = self.analyze_data(columns, rows);
        self.generate_config(&chart_type, columns, rows, &analysis)
    }
}

impl Default for ChartGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 数据分析结果
#[derive(Debug, Clone)]
struct DataAnalysis {
    row_count: usize,
    col_count: usize,
    numeric_cols: Vec<usize>,
    categorical_cols: Vec<usize>,
    date_cols: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_generator() -> ChartGenerator {
        ChartGenerator::new()
    }

    #[test]
    fn test_recommend_bar_chart() {
        let generator = create_test_generator();

        let columns = vec![
            ColumnMetadata { name: "category".to_string(), data_type: "varchar".to_string(), ordinal: 0 },
            ColumnMetadata { name: "value".to_string(), data_type: "int".to_string(), ordinal: 1 },
        ];

        let rows = vec![
            vec![serde_json::json!("A"), serde_json::json!(100)],
            vec![serde_json::json!("B"), serde_json::json!(200)],
            vec![serde_json::json!("C"), serde_json::json!(150)],
        ];

        let result = generator.recommend(&columns, &rows);
        assert!(result.is_ok());

        let recommendation = result.unwrap();
        assert!(matches!(recommendation.recommended, ChartType::Bar | ChartType::Pie));
    }

    #[test]
    fn test_recommend_line_chart() {
        let generator = create_test_generator();

        let columns = vec![
            ColumnMetadata { name: "date".to_string(), data_type: "date".to_string(), ordinal: 0 },
            ColumnMetadata { name: "sales".to_string(), data_type: "int".to_string(), ordinal: 1 },
        ];

        let rows = vec![
            vec![serde_json::json!("2024-01"), serde_json::json!(100)],
            vec![serde_json::json!("2024-02"), serde_json::json!(150)],
            vec![serde_json::json!("2024-03"), serde_json::json!(120)],
        ];

        let result = generator.recommend(&columns, &rows);
        assert!(result.is_ok());

        let recommendation = result.unwrap();
        assert!(matches!(recommendation.recommended, ChartType::Line));
    }

    #[test]
    fn test_switch_chart_type() {
        let generator = create_test_generator();

        let columns = vec![
            ColumnMetadata { name: "x".to_string(), data_type: "varchar".to_string(), ordinal: 0 },
            ColumnMetadata { name: "y".to_string(), data_type: "int".to_string(), ordinal: 1 },
        ];

        let rows = vec![
            vec![serde_json::json!("A"), serde_json::json!(100)],
            vec![serde_json::json!("B"), serde_json::json!(200)],
        ];

        let config = generator.switch_chart_type(&columns, &rows, "bar");
        assert!(config.is_ok());
        assert_eq!(config.unwrap().chart_type, "bar");
    }
}
