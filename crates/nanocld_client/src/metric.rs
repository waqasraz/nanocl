use nanocl_error::{
  io::IoError,
  http_client::{HttpClientResult, HttpClientError},
};

use nanocl_stubs::{
  metric::{Metric, MetricPartial},
  generic::{GenericFilter, GenericListQuery},
};

use super::http_client::NanocldClient;

impl NanocldClient {
  /// ## Default path for metrics
  const METRIC_PATH: &'static str = "/metrics";

  /// List existing metrics in the system
  ///
  /// ## Example
  ///
  /// ```no_run,ignore
  /// use nanocld_client::NanocldClient;
  ///
  /// let client = NanocldClient::connect_to("http://localhost:8585", None);
  /// let res = client.list_metric(None).await;
  /// ```
  pub async fn list_metric(
    &self,
    query: Option<&GenericFilter>,
  ) -> HttpClientResult<Vec<Metric>> {
    let query = query.cloned().unwrap_or_default();
    let query = GenericListQuery::try_from(query).map_err(|err| {
      HttpClientError::IoError(IoError::invalid_data(
        "Query".to_owned(),
        err.to_string(),
      ))
    })?;
    let res = self.send_get(Self::METRIC_PATH, Some(&query)).await?;
    Self::res_json(res).await
  }

  /// Create a new metric in the system
  ///
  /// ## Example
  ///
  /// ```no_run,ignore
  /// use nanocld_client::NanocldClient;
  /// use nanocld_client::stubs::metric::MetricPartial;
  ///
  /// let client = NanocldClient::connect_to("http://localhost:8585", None);
  /// let res = client.list_metric(&MetricPartial {
  ///  kind: "my-source.io/type".to_owned(),
  ///  data: serde_json::json!({
  ///   "name": "my-metric",
  ///   "description": "My metric",
  ///  }),
  /// }).await;
  /// ```
  pub async fn create_metric(
    &self,
    metric: &MetricPartial,
  ) -> HttpClientResult<Metric> {
    let res = self
      .send_post(Self::METRIC_PATH, Some(metric), None::<String>)
      .await?;
    Self::res_json(res).await
  }

  /// Inspect a metric in the system
  ///
  /// ## Example
  ///
  /// ```no_run,ignore
  /// use nanocld_client::NanocldClient;
  ///
  /// let client = NanocldClient::connect_to("http://localhost:8585", None);
  /// let res = client.inspect_metric("my-metric-key").await;
  /// ```
  pub async fn inspect_metric(&self, key: &str) -> HttpClientResult<Metric> {
    let res = self
      .send_get(
        &format!("{}/{key}/inspect", Self::METRIC_PATH),
        None::<String>,
      )
      .await?;
    Self::res_json(res).await
  }
}

#[cfg(test)]
mod tests {
  use crate::ConnectOpts;

  use super::*;

  #[ntex::test]
  async fn basic() {
    let client = NanocldClient::connect_to(&ConnectOpts {
      url: "http://nanocl.internal:8585".into(),
      ..Default::default()
    });
    let metric = client
      .create_metric(&MetricPartial {
        kind: "my-source.io/type".to_owned(),
        data: serde_json::json!({
          "name": "my-metric",
          "description": "My metric",
        }),
        note: None,
      })
      .await
      .unwrap();
    assert_eq!(metric.kind, "my-source.io/type");
    let metrics = client.list_metric(None).await.unwrap();
    assert!(!metrics.is_empty());
    client
      .inspect_metric(metrics[0].key.to_string().as_str())
      .await
      .unwrap();
  }
}
