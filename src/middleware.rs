use reqwest::{header, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use task_local_extensions::Extensions;

pub(crate) struct RetryMiddleware {
    inner: RetryTransientMiddleware<ExponentialBackoff>,
}

#[async_trait::async_trait]
impl Middleware for RetryMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        // check if req is cloneable without using try_clone
        // check request header - if content-type is multipart/form-data, then don't retry
        let Some(Ok(content_type)) = req.headers().get(header::CONTENT_TYPE).map(|v| v.to_str())
        else {
            return self.inner.handle(req, extensions, next).await;
        };

        if content_type.contains("multipart/form-data")
            || content_type == "application/octet-stream"
        {
            return next.run(req, extensions).await;
        }

        // what about other content types? But at least for OpenAI APIs, we only
        // see multipart/form-data as non-retryable
        return self.inner.handle(req, extensions, next).await;
    }
}

impl From<RetryTransientMiddleware<ExponentialBackoff>> for RetryMiddleware {
    fn from(inner: RetryTransientMiddleware<ExponentialBackoff>) -> Self {
        Self { inner }
    }
}
