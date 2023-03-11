use core::time::Duration;

use socket2::{Socket, TcpKeepalive};
use std::{io::Error as IoError, net::TcpStream as StdTcpStream};
use tokio::net::TcpStream;

#[cfg(unix)]
use std::os::fd::{FromRawFd as _, IntoRawFd as _};
#[cfg(windows)]
use std::os::windows::{FromRawSocket as _, IntoRawSocket as _};

//
#[cfg(any(unix, windows))]
pub fn tcp_stream_configure_keepalive(
    tcp_stream: TcpStream,
    time: Option<Duration>,
    interval: Option<Duration>,
    retries: Option<u32>,
) -> Result<TcpStream, IoError> {
    let tcp_keepalive = TcpKeepalive::new();

    let tcp_keepalive = if let Some(time) = time {
        tcp_keepalive.with_time(time)
    } else {
        tcp_keepalive
    };

    let tcp_keepalive = if let Some(interval) = interval {
        tcp_keepalive.with_interval(interval)
    } else {
        tcp_keepalive
    };

    #[allow(unused_variables)]
    let tcp_keepalive = if let Some(retries) = retries {
        #[cfg(windows)]
        {
            tcp_keepalive.with_retries(retries)
        }
        #[cfg(unix)]
        {
            tcp_keepalive
        }
    } else {
        tcp_keepalive
    };

    tcp_stream_configure(tcp_stream, move |socket| {
        socket.set_keepalive(true)?;
        socket.set_tcp_keepalive(&tcp_keepalive)?;
        Ok(socket)
    })
}

//
#[cfg(any(unix, windows))]
pub fn tcp_stream_configure<F>(tcp_stream: TcpStream, f: F) -> Result<TcpStream, IoError>
where
    F: Fn(Socket) -> Result<Socket, IoError>,
{
    let std_tcp_stream = tcp_stream.into_std()?;

    #[cfg(unix)]
    let socket = unsafe { Socket::from_raw_fd(std_tcp_stream.into_raw_fd()) };
    #[cfg(windows)]
    let socket = unsafe { Socket::from_raw_socket(std_tcp_stream.into_raw_socket()) };

    let socket = f(socket)?;

    #[cfg(unix)]
    let std_tcp_stream = unsafe { StdTcpStream::from_raw_fd(socket.into_raw_fd()) };
    #[cfg(windows)]
    let std_tcp_stream = unsafe { StdTcpStream::from_raw_socket(socket.into_raw_socket()) };

    TcpStream::from_std(std_tcp_stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(unix, windows))]
    #[tokio::test]
    async fn test_tcp_stream_configure_keepalive() {
        let tcp_stream = match TcpStream::connect("google.com:443").await {
            Ok(x) => x,
            Err(_) => return,
        };

        match tcp_stream_configure_keepalive(tcp_stream, Some(Duration::from_secs(15)), None, None)
        {
            Ok(_) => {}
            Err(err) => panic!("{err}"),
        }
    }
}
