use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{ok, Ready};
use log::debug;
use reqwest::Client;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};

// Define custom middleware with webhook URL
pub struct RequestNotifier {
    webhook_url: String,
    client: Arc<Client>,
}

impl RequestNotifier {
    pub fn new(webhook_url: String, client: Arc<Client>) -> Self {
        RequestNotifier {
            webhook_url,
            client,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestNotifier
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = RequestNotifierMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestNotifierMiddleware {
            service: Rc::new(service),
            webhook_url: self.webhook_url.clone(),
            client: self.client.clone(),
        })
    }
}

pub struct RequestNotifierMiddleware<S> {
    service: Rc<S>,
    webhook_url: String,
    client: Arc<Client>,
}

impl<S, B> Service<ServiceRequest> for RequestNotifierMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Capture required information from the request before it is moved.
        let fut = self.service.call(req);
        let webhook_url = self.webhook_url.clone();
        let client = self.client.clone();

        // Call the notifier asynchronously
        actix_rt::spawn(async move {
            // Send an empty POST request asynchronously to the webhook URL
            match client.post(&webhook_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Successfully sent log to webhook");
                    } else {
                        debug!("Failed to send log to webhook: {:?}", response.status());
                    }
                }
                Err(e) => {
                    debug!("Error sending log to webhook: {:?}", e);
                }
            }
        });

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
