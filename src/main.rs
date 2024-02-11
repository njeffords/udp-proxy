//! A simple UDP proxy server
//! 
//! This program listens on a local UDP socket and forwards all received packets
//! to a remote UDP socket. It also listens on the remote UDP socket and
//! forwards all received packets to the local UDP socket.

use std::{net::{ Ipv4Addr, SocketAddr}, sync::Arc};

use tokio::net::UdpSocket;

#[derive(clap::Parser)]
struct Cli {
    /// The local address to listen on
    src: SocketAddr,

    /// The remote address to forward to
    dst: SocketAddr,
}

/// The main function
/// 
/// It parses the command line arguments, creates a local UDP socket and a remote
/// UDP socket, and then forwards all received packets from the local socket to
/// the remote socket and vice versa.
#[tokio::main]
async fn main() {

    let Cli{ src, dst }: Cli = clap::Parser::parse();

    let local = Arc::new(UdpSocket::bind(src).await.unwrap());
    let remote = Arc::new(UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap());

    remote.connect(dst).await.unwrap();

    let mut buf = [0; 2048];

    let (size, addr) = local.recv_from(&mut buf).await.unwrap();
    remote.send(&buf[..size]).await.unwrap();

    local.connect(addr).await.unwrap();

    {
        let local = local.clone();
        let remote = remote.clone();

        tokio::spawn(async move {
            let mut buf = [0; 2048];
            loop {
                let size = remote.recv(&mut buf).await.unwrap();
                local.send(&buf[..size]).await.unwrap();
            }
        });
    }

    loop {
        let size = local.recv(&mut buf).await.unwrap();
        remote.send(&buf[..size]).await.unwrap();
    }

}
