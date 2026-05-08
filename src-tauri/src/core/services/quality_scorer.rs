use crate::core::services::result_service::{
    ColumnInsightFull, ColumnQualityEntry, ColumnStatsDetail, QualityDimension, QualityScore,
    TableQuality,
};

pub(crate) fn compute_column_quality(stats: &ColumnInsightFull) -> QualityScore {
    let null_rate = stats.stats.null_rate;
    let total = stats.stats.total_count as f64;
    let unique = stats.stats.unique_count.unwrap_or(0) as f64;
    let non_null = total * (1.0 - null_rate);

    let completeness = if total > 0.0 {
        (1.0 - null_rate) * 100.0
    } else {
        0.0
    };

    let uniqueness = if non_null > 0.0 {
        let ratio = unique / non_null;
        if ratio > 0.9 {
            100.0
        } else if ratio > 0.5 {
            80.0
        } else if ratio > 0.2 {
            60.0
        } else if ratio > 0.05 {
            40.0
        } else if ratio > 0.01 {
            20.0
        } else {
            10.0
        }
    } else {
        0.0
    };

    let type_consistency = match stats.stats.stats_detail {
        ColumnStatsDetail::Numeric(_) => {
            if null_rate > 0.5 {
                40.0
            } else {
                90.0
            }
        }
        ColumnStatsDetail::Text(_) => {
            if unique < 2.0 {
                30.0
            } else if null_rate > 0.6 {
                40.0
            } else {
                75.0
            }
        }
        ColumnStatsDetail::DateTime(_) => {
            let has_range = stats
                .histogram
                .as_ref()
                .map_or(false, |h| h.len() > 1);
            if has_range {
                85.0
            } else {
                60.0
            }
        }
        ColumnStatsDetail::Boolean(_) => 95.0,
        ColumnStatsDetail::Unknown => 50.0,
    };

    fn detail_variant_name(detail: &ColumnStatsDetail) -> &str {
        match detail {
            ColumnStatsDetail::Numeric(_) => "Numeric",
            ColumnStatsDetail::Text(_) => "Text",
            ColumnStatsDetail::DateTime(_) => "DateTime",
            ColumnStatsDetail::Boolean(_) => "Boolean",
            ColumnStatsDetail::Unknown => "Unknown",
        }
    }

    let distribution = if let Some(ref hist) = stats.histogram {
        let bins = hist.len() as f64;
        if bins > 0.0 {
            let values: Vec<f64> = hist.iter().map(|b| b.count as f64).collect();
            let sum: f64 = values.iter().sum();
            if sum > 0.0 {
                let avg = sum / bins;
                let variance: f64 =
                    values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / bins;
                let cv = variance.sqrt() / avg.max(1.0);
                if cv < 0.3 {
                    90.0
                } else if cv < 0.7 {
                    75.0
                } else if cv < 1.5 {
                    50.0
                } else {
                    30.0
                }
            } else {
                50.0
            }
        } else {
            50.0
        }
    } else {
        50.0
    };

    let unique_display = stats.stats.unique_count.unwrap_or(0);

    let dimensions = vec![
        QualityDimension {
            name: "完整性".into(),
            score: completeness,
            weight: 0.35,
            detail: format!("空值率 {:.1}%", null_rate * 100.0),
        },
        QualityDimension {
            name: "唯一性".into(),
            score: uniqueness,
            weight: 0.25,
            detail: format!("去重 {}/{}", unique_display, stats.stats.total_count),
        },
        QualityDimension {
            name: "类型一致".into(),
            score: type_consistency,
            weight: 0.20,
            detail: detail_variant_name(&stats.stats.stats_detail).into(),
        },
        QualityDimension {
            name: "分布均匀".into(),
            score: distribution,
            weight: 0.20,
            detail: "直方图分布评估".into(),
        },
    ];

    let overall: f64 = dimensions.iter().map(|d| d.score * d.weight).sum();

    let level = if overall >= 85.0 {
        "优秀"
    } else if overall >= 70.0 {
        "良好"
    } else if overall >= 50.0 {
        "一般"
    } else if overall >= 30.0 {
        "较差"
    } else {
        "差"
    };

    let summary = if overall >= 85.0 {
        format!("数据质量优秀 ({:.0}分)，可直接用于分析", overall)
    } else if overall >= 70.0 {
        format!("数据质量良好 ({:.0}分)，建议关注空值", overall)
    } else if overall >= 50.0 {
        format!("数据质量一般 ({:.0}分)，存在明显质量问题", overall)
    } else {
        format!("数据质量较差 ({:.0}分)，建议清洗后使用", overall)
    };

    QualityScore {
        column_name: stats.stats.column_name.clone(),
        overall_score: overall,
        level: level.into(),
        dimensions,
        summary,
    }
}

pub(crate) fn compute_table_quality(
    table_name: &str,
    stats_list: &[ColumnInsightFull],
) -> TableQuality {
    let mut entries: Vec<ColumnQualityEntry> = stats_list
        .iter()
        .map(|s| {
            let qs = compute_column_quality(s);
            ColumnQualityEntry {
                column_name: s.stats.column_name.clone(),
                quality_score: qs.overall_score,
                level: qs.level,
                null_rate: s.stats.null_rate,
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        a.quality_score
            .partial_cmp(&b.quality_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let scored_count = entries.len();
    let total_columns = scored_count;
    let overall = if scored_count > 0 {
        entries.iter().map(|e| e.quality_score).sum::<f64>() / scored_count as f64
    } else {
        0.0
    };

    let level = if overall >= 85.0 {
        "优秀"
    } else if overall >= 70.0 {
        "良好"
    } else if overall >= 50.0 {
        "一般"
    } else if overall >= 30.0 {
        "较差"
    } else {
        "差"
    };

    let problem_columns = entries
        .iter()
        .filter(|e| e.quality_score < 50.0)
        .count();
    let summary = if scored_count == 0 {
        "无数据".into()
    } else if overall >= 85.0 {
        format!("表质量优秀 ({:.0}分)，{} 列均健康", overall, scored_count)
    } else if problem_columns > 0 {
        format!(
            "表质量{} ({:.0}分)，{} 列需关注 ({}风险列)",
            level, overall, scored_count, problem_columns
        )
    } else {
        format!(
            "表质量{} ({:.0}分)，{} 列已评估",
            level, overall, scored_count
        )
    };

    TableQuality {
        table_name: table_name.into(),
        overall_score: overall,
        level: level.into(),
        column_scores: entries,
        summary,
        scored_count,
        total_columns,
    }
}
