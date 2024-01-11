use crate::{error::ServerFnError, request::ClientReq, response::ClientRes};
use std::future::Future;

/// A client defines a pair of request/response types and the logic to send
/// and receive them.
///
/// This trait is implemented for things like a browser `fetch` request or for
/// the `reqwest` trait. It should almost never be necessary to implement it
/// yourself, unless you’re trying to use an alternative HTTP crate on the client side.
pub trait Client<CustErr> {
    /// The type of a request sent by this client.
    type Request: ClientReq<CustErr> + Send;
    /// The type of a response received by this client.
    type Response: ClientRes<CustErr> + Send;

    /// Sends the request and receives a response.
    fn send(
        req: Self::Request,
    ) -> impl Future<Output = Result<Self::Response, ServerFnError<CustErr>>> + Send;
}

#[cfg(any(feature = "browser", doc))]
/// Implements [`Client`] for a `fetch` request in the browser.
pub mod browser {
    use super::Client;
    use crate::{
        error::ServerFnError, request::browser::BrowserRequest,
        response::browser::BrowserResponse,
    };
    use send_wrapper::SendWrapper;
    use std::future::Future;

    /// Implements [`Client`] for a `fetch` request in the browser.    
    pub struct BrowserClient;

    impl<CustErr> Client<CustErr> for BrowserClient {
        type Request = BrowserRequest;
        type Response = BrowserResponse;

        fn send(
            req: Self::Request,
        ) -> impl Future<Output = Result<Self::Response, ServerFnError<CustErr>>>
               + Send {
            SendWrapper::new(async move {
                req.0
                    .take()
                    .send()
                    .await
                    .map(|res| BrowserResponse(SendWrapper::new(res)))
                    .map_err(|e| ServerFnError::Request(e.to_string()))
            })
        }
    }
}

#[cfg(any(feature = "reqwest", doc))]
pub mod reqwest {
    use super::Client;
    use crate::{error::ServerFnError, request::reqwest::CLIENT};
    use futures::TryFutureExt;
    use reqwest::{Request, Response};
    use std::future::Future;

    pub struct ReqwestClient;

    impl<CustErr> Client<CustErr> for ReqwestClient {
        type Request = Request;
        type Response = Response;

        fn send(
            req: Self::Request,
        ) -> impl Future<Output = Result<Self::Response, ServerFnError<CustErr>>>
               + Send {
            CLIENT
                .execute(req)
                .map_err(|e| ServerFnError::Request(e.to_string()))
        }
    }
}
