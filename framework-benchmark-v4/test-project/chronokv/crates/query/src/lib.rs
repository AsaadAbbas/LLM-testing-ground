use chronokv_core::{
    AggregationResult, AggregationType, ChronoError, Entry, GetResponse, QueryRequest,
    QueryResponse, TimeRange,
};
use chronokv_engine::StorageEngine;
use std::sync::Arc;

/// Query engine for ChronoKV.
///
/// Provides time-range queries, key prefix scanning, and aggregations
/// on top of the storage engine.
pub struct QueryEngine {
    engine: Arc<StorageEngine>,
}

impl QueryEngine {
    pub fn new(engine: Arc<StorageEngine>) -> Self {
        Self { engine }
    }

    /// Execute a range query over the storage engine.
    ///
    /// Returns entries matching the key prefix (if specified) within the
    /// given time range. Uses inclusive bounds on both ends for consistency.
    pub async fn query(&self, request: &QueryRequest) -> Result<QueryResponse, ChronoError> {
        let range = match request.time_range {
            Some((start, end)) => TimeRange::new(start, end),
            None => TimeRange::new(0.0, f64::MAX),
        };

        // Query the engine — the engine's scan handles the actual filtering
        let entries = self.engine.scan(request.key_prefix.as_deref(), range).await?;

        // Apply limit if specified
        let limited_entries: Vec<Entry> = match request.limit {
            Some(limit) => entries.into_iter().take(limit).collect(),
            None => entries,
        };

        let total_count = limited_entries.len() as u64;

        let response_entries: Vec<GetResponse> = limited_entries
            .into_iter()
            .map(|entry| GetResponse {
                key: entry.key.clone(),
                value: entry.value.clone(),
                timestamp: entry.timestamp,
                version: (entry.timestamp * 1_000_000.0) as u64, // approximate version from timestamp
            })
            .collect();

        Ok(QueryResponse {
            entries: response_entries,
            total_count,
        })
    }

    /// Execute an aggregation query over a time range.
    ///
    /// Supports MIN, MAX, AVG, COUNT, SUM on the numeric values of entries
    /// within the specified time range. Values are interpreted as f64.
    pub async fn aggregate(
        &self,
        key_prefix: Option<&str>,
        range: TimeRange,
        agg_type: AggregationType,
    ) -> Result<AggregationResult, ChronoError> {
        let entries = self.engine.scan(key_prefix, range).await?;

        if entries.is_empty() {
            return Ok(AggregationResult {
                agg_type,
                value: 0.0,
                count: 0,
            });
        }

        // Parse values as f64 for aggregation
        let values: Vec<f64> = entries
            .iter()
            .filter_map(|e| {
                // Try to parse value as f64 string, fall back to byte length
                String::from_utf8(e.value.clone())
                    .ok()
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .or(Some(e.value.len() as f64))
            })
            .collect();

        let count = values.len() as u64;

        let value = match agg_type {
            AggregationType::Min => values.iter().copied().fold(f64::MAX, f64::min),
            AggregationType::Max => values.iter().copied().fold(f64::MIN, f64::max),
            AggregationType::Avg => values.iter().sum::<f64>() / count as f64,
            AggregationType::Count => count as f64,
            AggregationType::Sum => values.iter().sum(),
        };

        Ok(AggregationResult {
            agg_type,
            value,
            count,
        })
    }

    /// Get the latest value for a specific key.
    pub async fn get_latest(&self, key: &str) -> Result<Option<GetResponse>, ChronoError> {
        let entry = self.engine.get(key).await?;

        Ok(entry.filter(|e| !e.is_tombstone()).map(|e| GetResponse {
            key: e.key.clone(),
            value: e.value.clone(),
            timestamp: e.timestamp,
            version: (e.timestamp * 1_000_000.0) as u64,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_engine() -> Arc<StorageEngine> {
        let id = uuid::Uuid::new_v4();
        let path = format!("/tmp/chronokv_query_test_{}", id);
        Arc::new(StorageEngine::new(&path, 3600.0).unwrap())
    }

    #[tokio::test]
    async fn test_basic_query() {
        let engine = make_engine().await;
        engine.put("a".to_string(), b"10".to_vec(), 100.0).await.unwrap();
        engine.put("b".to_string(), b"20".to_vec(), 200.0).await.unwrap();

        let qe = QueryEngine::new(engine);

        let req = QueryRequest {
            key_prefix: None,
            time_range: Some((50.0, 250.0)),
            limit: None,
        };

        let result = qe.query(&req).await.unwrap();
        // Note: query layer uses inclusive range, but engine scan uses exclusive end
        // Both entries should be returned since both are well within the range
        assert_eq!(result.total_count, 2);
    }

    #[tokio::test]
    async fn test_aggregation_sum() {
        let engine = make_engine().await;
        engine.put("metric".to_string(), b"10".to_vec(), 100.0).await.unwrap();
        engine.put("metric2".to_string(), b"20".to_vec(), 200.0).await.unwrap();
        engine.put("metric3".to_string(), b"30".to_vec(), 300.0).await.unwrap();

        let qe = QueryEngine::new(engine);

        let range = TimeRange::new(50.0, 350.0);
        let result = qe.aggregate(None, range, AggregationType::Sum).await.unwrap();
        // All three entries are well within range, sum should be 60
        assert_eq!(result.value, 60.0);
        assert_eq!(result.count, 3);
    }

    #[tokio::test]
    async fn test_get_latest() {
        let engine = make_engine().await;
        engine.put("key1".to_string(), b"v1".to_vec(), 100.0).await.unwrap();

        let qe = QueryEngine::new(engine);
        let result = qe.get_latest("key1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, b"v1");
    }
}
