use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;

const PUBLIC_KEY_PATH: &str = "/Users/radi/.ssh/id_rsa.pub";
const PRIVATE_KEY_PATH: &str = "/Users/radi/.ssh/id_rsa";
const REMOTE_LOG_PATH: &str = "/var/log/pihole.log";

fn establish_connection() {
    let tcp = TcpStream::connect("192.168.1.10:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    let public_key_path = Path::new(PUBLIC_KEY_PATH);
    let private_key_path = Path::new(PRIVATE_KEY_PATH);
    sess.userauth_pubkey_file("pi", Some(public_key_path), private_key_path, None)
        .unwrap();

    assert!(sess.authenticated());
}
