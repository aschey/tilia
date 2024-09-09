use std::io;
use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use futures::Future;
use transport_async::codec::LengthDelimitedCodec;
use transport_async::Connect;

use crate::BoxedError;

#[cfg(feature = "ipc")]
pub fn ipc_client(
    name: impl transport_async::ipc::IntoIpcPath + Clone + 'static,
) -> impl Fn() -> Pin<
    Box<
        dyn Future<
                Output = Result<
                    transport_async::codec::EncodedStream<BytesMut, Bytes, io::Error, io::Error>,
                    BoxedError,
                >,
            > + Send,
    >,
> + Clone
+ Send {
    let name = name.clone();

    move || {
        let name = name.clone();
        Box::pin(async move {
            let client_transport = transport_async::ipc::Connection::connect(
                transport_async::ipc::ConnectionParams::new(name)?,
            )
            .await?;
            Ok(LengthDelimitedCodec::client(client_transport))
        })
    }
}

#[cfg(feature = "tcp")]
pub fn tcp_client(
    addr: impl tokio::net::ToSocketAddrs + Clone + Send + Sync + 'static,
) -> impl Fn() -> Pin<
    Box<
        dyn Future<
                Output = Result<
                    transport_async::codec::EncodedStream<BytesMut, Bytes, io::Error, io::Error>,
                    BoxedError,
                >,
            > + Send,
    >,
> + Clone
+ Send {
    move || {
        let addr = addr.clone();
        Box::pin(async move {
            let client_transport = transport_async::tcp::Connection::connect(addr).await?;
            Ok(LengthDelimitedCodec::client(client_transport))
        })
    }
}

#[cfg(feature = "docker")]
pub mod docker {
    use std::pin::Pin;
    use std::task::Poll;

    use background_service::error::BoxedError;
    use bollard::container::{LogOutput, LogsOptions};
    use bollard::Docker;
    use bytes::BytesMut;
    use futures::{Future, Stream};
    use pin_project_lite::pin_project;

    pub type DockerLogStream = Pin<Box<dyn Future<Output = Result<LogStream, BoxedError>> + Send>>;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum LogSource {
        Stdout,
        Stderr,
        All,
    }

    pub fn docker_client(
        container: impl Into<String>,
        log_source: LogSource,
    ) -> impl Fn() -> DockerLogStream + Clone + Send {
        let container = container.into();
        move || {
            let docker = Docker::connect_with_local_defaults().unwrap();

            let logs = docker.logs(
                &container.clone(),
                Some(LogsOptions::<String> {
                    stderr: matches!(log_source, LogSource::Stderr | LogSource::All),
                    stdout: matches!(log_source, LogSource::Stdout | LogSource::All),
                    follow: true,
                    ..Default::default()
                }),
            );
            let stream = LogStream {
                inner: Box::pin(logs),
            };
            Box::pin(async move { Ok(stream) })
        }
    }

    pin_project! {
        pub struct LogStream
        {
            #[pin]
            inner: Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>,
        }
    }

    impl Stream for LogStream {
        type Item = Result<BytesMut, bollard::errors::Error>;

        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            match self.project().inner.poll_next(cx) {
                std::task::Poll::Ready(Some(Ok(log))) => Poll::Ready(Some(Ok(BytesMut::from(
                    // replace newlines to fix wonky formatting
                    log.to_string().replace('\n', " ").as_bytes(),
                )))),
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}
