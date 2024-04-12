use std::io;
use std::pin::Pin;

use bytes::{Bytes, BytesMut};
use futures::Future;
use tower::BoxError;
use tower_rpc::{length_delimited_codec, transport, CodecStream};

#[cfg(feature = "ipc")]
pub fn ipc_client(
    name: impl tower_rpc::transport::ipc::IntoIpcPath + Clone + 'static,
) -> impl Fn() -> Pin<
    Box<
        dyn Future<Output = Result<CodecStream<BytesMut, Bytes, io::Error, io::Error>, BoxError>>
            + Send,
    >,
> + Clone
+ Send {
    let name = name.clone();
    move || {
        let name = name.clone();
        Box::pin(async move {
            let ipc_path = name.into_ipc_path()?;
            let client_transport = transport::ipc::connect(ipc_path).await?;
            Ok(length_delimited_codec(client_transport))
        })
    }
}

#[cfg(feature = "tcp")]
pub fn tcp_client(
    addr: impl tokio::net::ToSocketAddrs + Clone + Send + Sync + 'static,
) -> impl Fn() -> Pin<
    Box<
        dyn Future<Output = Result<CodecStream<BytesMut, Bytes, io::Error, io::Error>, BoxError>>
            + Send,
    >,
> + Clone
+ Send {
    move || {
        let addr = addr.clone();
        Box::pin(async move {
            let client_transport = transport::tcp::connect(addr.clone()).await?;
            Ok(length_delimited_codec(client_transport))
        })
    }
}

#[cfg(feature = "docker")]
pub mod docker {
    use std::convert::Infallible;
    use std::pin::Pin;
    use std::task::Poll;

    use bollard::container::{LogOutput, LogsOptions};
    use bollard::Docker;
    use bytes::{Bytes, BytesMut};
    use futures::{Future, Sink, Stream};
    use pin_project_lite::pin_project;
    use tower::BoxError;

    pub type DockerLogStream = Pin<Box<dyn Future<Output = Result<LogStream, BoxError>> + Send>>;

    pub fn docker_client(
        container: impl Into<String>,
    ) -> impl Fn() -> DockerLogStream + Clone + Send {
        let container = container.into();
        move || {
            let docker = Docker::connect_with_local_defaults().unwrap();

            let logs = docker.logs(
                &container.clone(),
                Some(LogsOptions::<String> {
                    stderr: true,
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
                    log.into_bytes().to_vec().as_slice(),
                )))),
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }

    impl Sink<Bytes> for LogStream {
        type Error = Infallible;

        fn poll_ready(
            self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, _item: Bytes) -> Result<(), Self::Error> {
            Ok(())
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }
}
