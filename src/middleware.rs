use std::time::Instant;

use poem::{Endpoint, IntoResponse, Middleware, Request, Response, Result};

/// Middleware for [`tracing`](https://crates.io/crates/tracing).
#[derive(Default)]
pub struct HttpMetricMiddleware;

impl<E: Endpoint> Middleware<E> for HttpMetricMiddleware {
    type Output = HttpMetricMiddlewareEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        HttpMetricMiddlewareEndpoint { inner: ep }
    }
}

/// Endpoint for `Tracing` middleware.
pub struct HttpMetricMiddlewareEndpoint<E> {
    inner: E,
}

impl<E: Endpoint> Endpoint for HttpMetricMiddlewareEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        let now = Instant::now();
        let res = self.inner.call(req).await;
        let latency = now.elapsed();

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                metrics::counter!("http_requests_total").increment(1);
                metrics::histogram!("http_requests_duration_seconds").record(latency.as_secs_f64());
                Ok(resp)
            }
            Err(err) => {
                metrics::counter!("http_requests_error").increment(1);
                Err(err)
            }
        }
    }
}
