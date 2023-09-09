use std::{
    fs::{self, File},
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread::JoinHandle,
    time::Instant,
};
extern crate bytesize;
use bytesize::ByteSize;

fn transfer(mut stream: TcpStream, addr: SocketAddr) {
    let mut name_len_buf = [0u8; 2];
    stream
        .read_exact(&mut name_len_buf)
        .expect("file name length");
    let mut file_name_buf = vec![0; u16::from_be_bytes(name_len_buf) as usize];
    stream.read_exact(&mut file_name_buf).expect("file name");
    let file_name = String::from_utf8(file_name_buf).expect("correct utf8 name");
    let mut len_buf = [0u8; 8];
    stream.read_exact(&mut len_buf).expect("file size");
    let file_size = u64::from_be_bytes(len_buf);

    let mut file = File::create(format!("uploads/{}", file_name)).expect("Correct file");
    let mut buf = [0u8; 4096];
    let mut transfered_size = 0usize;
    let mut transfered_3_secs = 0usize;
    let global_time = Instant::now();
    let mut time = Instant::now();
    loop {
        match stream.read(&mut buf) {
            Ok(n) => {
                if n > 0 {
                    file.write_all(&buf[..n]).expect("file part write");
                    transfered_3_secs += n;
                } else {
                    break;
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::Interrupted => {}
                _ => {
                    println!("Error while transfering {}...", file_name);
                    return;
                }
            },
        }
        let time_elapsed = time.elapsed().as_secs_f32();
        let time_elapsed_global = global_time.elapsed().as_secs_f32();
        if time_elapsed >= 3. {
            transfered_size += transfered_3_secs;
            println!(
                "Transfer speed for {} from {} is {}/s average is {}/s",
                file_name,
                addr,
                ByteSize::b((transfered_3_secs as f32 / time_elapsed) as u64),
                ByteSize::b((transfered_size as f32 / time_elapsed_global) as u64)
            );
            transfered_3_secs = 0;
            time = Instant::now();
        }
    }
    let time_elapsed_global = global_time.elapsed().as_secs_f32();
    print!(
        "Transfer speed for {} from {} is {}/s",
        file_name,
        addr,
        ByteSize::b((transfered_3_secs as f32 / time.elapsed().as_secs_f32()) as u64)
    );
    transfered_size += transfered_3_secs;
    if time_elapsed_global > 3. {
        println!(
            " average is {}/s",
            ByteSize::b((transfered_size as f32 / time_elapsed_global) as u64)
        );
    } else {
        println!();
    }
    stream
        .write_all(&[(transfered_size as u64 == file_size) as u8])
        .expect("successful answer");
    stream.flush().expect("successful flush");
    println!("file {} successfully transfered", file_name);
}

fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .or_else(|| {
            println!("Port was not passed. Using 48666...");
            Some(String::from("48666"))
        })
        .expect("Valid argument or default")
        .parse()
        .expect("Valid port string");

    let _ = fs::create_dir("uploads");

    let listener = TcpListener::bind(("0.0.0.0", port)).expect("bound TcpListener");
    let mut connections: Vec<JoinHandle<()>> = vec![];
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("new connection from {}", addr);
                connections.push(std::thread::spawn(move || transfer(stream, addr)))
            }
            Err(e) => {
                println!("Error while accepting: {}. exiting...", e);
                break;
            }
        }
    }
    for handle in connections {
        let _ = handle.join();
    }
}
