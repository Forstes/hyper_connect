use tokio_socks::tcp::Socks5Stream;

pub async fn connect_to_socks_proxy(
    address: &str,
    username: &str,
    password: &str,
) -> Result<Socks5Stream<TcpStream>, tokio_socks::Error> {
    let socket = TcpStream::connect(address).await?;
    Socks5Stream::connect_with_password_and_socket(socket, address, username, password).await
}

/*         let stream: Socks5Stream<TcpStream> =
    connect_to_socks_proxy("207.244.217.165:6712", "dxjhvrqm", "1p487vhyxa6s").await?;
let stream = TokioIo::new(stream); */
