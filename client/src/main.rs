use std::{
    env,
    fs::File,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    os::unix::prelude::OsStrExt,
    path::Path,
};

fn main() {
    let mut args = env::args();
    args.next();
    let file_path_string = args.next().expect("file name at first argument");
    let address = args.next().expect("server address at second argument");

    let file_path = Path::new(file_path_string.as_str());
    let mut file = File::open(file_path).expect("valid file path");

    let mut resolved = address.to_socket_addrs().expect("valid address");
    let address = resolved.next().expect("existing address");
    let mut stream = TcpStream::connect(address).expect("successful connection");
    let file_name = file_path.file_name().expect("valid file name");
    let name_len_buf = (file_name.len() as u16).to_be_bytes();
    stream
        .write_all(&name_len_buf)
        .expect("Successful file name len write");
    stream
        .write_all(file_name.as_bytes())
        .expect("Successful file name write");
    let file_size = file
        .metadata()
        .expect("file has metadata")
        .len()
        .to_be_bytes();
    stream
        .write_all(&file_size)
        .expect("Successful file size write");

    let mut buffer = [0u8; 4096];
    loop {
        match file.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    stream.write_all(&buffer[..n]).expect("Successful write");
                } else {
                    stream.flush().expect("flush before closing write");
                    stream
                        .shutdown(std::net::Shutdown::Write)
                        .expect("Successful shutdown write");
                    break;
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Interrupted => {}
                _ => break,
            },
        };
    }
    let mut ans_buf = [0u8];
    stream.read_exact(&mut ans_buf).expect("Server answer");
    if ans_buf[0] == 1 {
        println!("File has been transfered successfully!");
    } else {
        println!("File has been transfered with errors");
    }
}
