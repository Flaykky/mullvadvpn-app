//! tokio based native IPC communication:
//! - Unix domain socket on Linux/macOS
//! - Named pipes on Windows

use std::{path::Path, pin::Pin};

use futures::{ready, Stream};
use tokio::io::{AsyncRead, AsyncWrite};

pub use std::io::Result;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{NamedPipeClient, NamedPipeServer};

#[cfg(unix)]
pub struct IpcEndpoint {
    path: String,
}

#[cfg(windows)]
pub struct IpcEndpoint {
    path: String,
    /// Only one named pipe can be created with the given name at a time.
    created: bool,
}

#[cfg(windows)]
impl IpcEndpoint {
    /// New IPC endpoint at the given path.
    pub fn new(path: String) -> Self {
        IpcEndpoint {
            path,
            created: false,
        }
    }

    pub fn incoming(
        mut self,
    ) -> Result<impl Stream<Item = Result<impl AsyncRead + AsyncWrite>> + 'static> {
        let pipe = self.create_listener()?;

        let stream =
            futures::stream::try_unfold((pipe, self), |(listener, mut endpoint)| async move {
                let () = listener.connect().await?;
                let new_listener = endpoint.create_listener()?;
                let conn = Connection::Server(listener);

                Ok(Some((conn, (new_listener, endpoint))))
            });

        Ok(stream)
    }

    pub async fn connect<P: AsRef<Path>>(pipe_name: P) -> Result<Connection> {
        use tokio::net::windows::named_pipe::ClientOptions;
        use tokio::time::{sleep, Duration, Instant};
        use windows_sys::Win32::Foundation::ERROR_PIPE_BUSY;

        let pipe_name = pipe_name.as_ref();

        let client = {
            const PIPE_AVAILABILITY_TIMEOUT: Duration = Duration::from_secs(5);
            let busy = |e: &std::io::Error| e.raw_os_error() == Some(ERROR_PIPE_BUSY as i32);
            let start = Instant::now();
            let unresponsive = |e: &_| busy(e) && start.elapsed() > PIPE_AVAILABILITY_TIMEOUT;

            loop {
                match ClientOptions::new().read(true).write(true).open(pipe_name) {
                    // Connected to a matching server
                    Ok(client) => break client,
                    // There is a server, but it has not has not served us within a reasonable timeframe.
                    Err(e) if unresponsive(&e) => return Err(e),
                    // There is a server, but it is currently busy. Sleep a little bit and try again
                    Err(e) if busy(&e) => sleep(Duration::from_millis(50)).await,
                    // There is (most likely) no server to connect to
                    Err(e) => return Err(e),
                }
            }
        };

        let conn = Connection::Client(client);
        Ok(conn)
    }

    fn create_listener(&mut self) -> Result<NamedPipeServer> {
        use tokio::net::windows::named_pipe::ServerOptions;
        let first = !self.created;
        let server = ServerOptions::new().first_pipe_instance(first)
            // Only allow local clients
            .reject_remote_clients(true)
            // Bi-directional
            .access_inbound(true)
            .access_outbound(true)
            .in_buffer_size(65536)
            .out_buffer_size(65536)
            .create(&self.path)?;
        self.created = true;
        Ok(server)
    }
}

#[cfg(unix)]
impl IpcEndpoint {
    /// New IPC endpoint at the given path.
    pub fn new(path: String) -> Self {
        IpcEndpoint { path }
    }

    pub fn incoming(
        self,
    ) -> Result<impl Stream<Item = Result<impl AsyncRead + AsyncWrite>> + 'static> {
        use nix::sys::stat::{fchmod, Mode};

        let uds = tokio::net::UnixListener::bind(&self.path)?;
        // TODO: Security attributes?
        // Change permissions on UDS
        const MODE: Mode = Mode::from_bits(0o766).unwrap();
        fchmod(&uds, MODE).unwrap();
        let incoming = Incoming {
            path: self.path.clone(),
            listener: uds,
        };
        Ok(incoming)
    }

    pub async fn connect<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let uds = tokio::net::UnixStream::connect(path).await?;
        Ok(Connection(uds))
    }
}

#[cfg(unix)]
struct Incoming {
    path: String,
    listener: tokio::net::UnixListener,
}

#[cfg(unix)]
impl Stream for Incoming {
    type Item = Result<tokio::net::UnixStream>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let result = ready!(self.listener.poll_accept(cx));
        let stream = result.map(|(stream, _addr)| stream);
        std::task::Poll::Ready(Some(stream))
    }
}

impl Drop for Incoming {
    // Remove the UDS on drop
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(unix)]
pub struct Connection(tokio::net::UnixStream);

impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        Pin::new(&mut this.0).poll_shutdown(cx)
    }
}

#[cfg(windows)]
pub enum Connection {
    Client(NamedPipeClient),
    Server(NamedPipeServer),
}

#[cfg(windows)]
impl AsyncRead for Connection {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = Pin::into_inner(self);
        match this {
            Connection::Client(ref mut client) => Pin::new(client).poll_read(cx, buf),
            Connection::Server(ref mut server) => Pin::new(server).poll_read(cx, buf),
        }
    }
}

#[cfg(windows)]
impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        let this = Pin::into_inner(self);
        match this {
            Connection::Client(ref mut client) => Pin::new(client).poll_write(cx, buf),
            Connection::Server(ref mut server) => Pin::new(server).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        match this {
            Connection::Client(ref mut client) => Pin::new(client).poll_flush(cx),
            Connection::Server(ref mut server) => Pin::new(server).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        let this = Pin::into_inner(self);
        match this {
            Connection::Client(ref mut client) => Pin::new(client).poll_shutdown(cx),
            Connection::Server(ref mut server) => Pin::new(server).poll_shutdown(cx),
        }
    }
}
