use std::env;
use std::io;
use std::io::Error;
use std::net;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::thread;

struct Server {
    local_address: net::SocketAddr,
    remote_address: net::SocketAddr,
}

impl Server {
    fn new(local_address: net::SocketAddr, remote_address: net::SocketAddr) -> Self {
        return Server {
            local_address,
            remote_address,
        };
    }

    fn run(self) {
        let tcp_listener = net::TcpListener::bind(self.local_address).unwrap();
        for mut tcp_stream in tcp_listener.incoming() {
            thread::spawn(move || {
                let dst_tcp_stream = net::TcpStream::connect(self.remote_address).unwrap();
                forward(dst_tcp_stream, tcp_stream.unwrap())
            });
        }
    }
}

fn main() {
    let matches = clap::App::new("forwarder")
        .version("0.1.0")
        .arg(
            clap::Arg::with_name("local_address")
                .long("local_address")
                .default_value("127.0.0.1:1234")
                .takes_value(true)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("remote_address")
                .long("remote_address")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let server = Server::new(
        net::SocketAddr::from_str(matches.value_of("local_address").unwrap()).unwrap(),
        net::SocketAddr::from_str(matches.value_of("remote_address").unwrap()).unwrap(),
    );
    server.run();
}

fn forward(dst_tcp_stream: net::TcpStream, src_tcp_stream: net::TcpStream) {
    let (mut src_reader, mut src_writer) = (
        src_tcp_stream.try_clone().unwrap(),
        src_tcp_stream.try_clone().unwrap(),
    );
    let (mut dst_reader, mut dst_writer) = (
        dst_tcp_stream.try_clone().unwrap(),
        dst_tcp_stream.try_clone().unwrap(),
    );
    let threads = vec![
        thread::spawn(move || match io::copy(&mut src_reader, &mut dst_writer) {
            _ => {}
        }),
        thread::spawn(move || match io::copy(&mut dst_reader, &mut src_writer) {
            _ => {}
        }),
    ];
    for thread in threads {
        thread.join().unwrap();
        match src_tcp_stream.shutdown(net::Shutdown::Both) {
            _ => {}
        }
        match dst_tcp_stream.shutdown(net::Shutdown::Both) {
            _ => {}
        }
    }
}
