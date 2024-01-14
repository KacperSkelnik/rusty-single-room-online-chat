use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

#[derive(Debug, Clone)]
struct Message {
    user_name: String,
    message: String,
    address: SocketAddr,
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.user_name, self.message)
    }
}

async fn process_single_connection(
    mut socket: TcpStream,
    address: SocketAddr,
    sender: Sender<Message>,
) {
    let mut receiver = sender.subscribe();

    let (read, mut write) = socket.split();
    let mut reader = BufReader::new(read);
    let mut line = String::new();

    let user_name = reader
        .read_line(&mut line)
        .await
        .map(|_| {
            let user_name = line.clone();
            line.clear();
            user_name
        })
        .unwrap();

    loop {
        tokio::select! {
            bytes_read = reader.read_line(&mut line) => {
                if bytes_read.unwrap() == 0 { break; }
                sender
                    .send(Message{ user_name: user_name.trim().to_string(), message: line.clone(), address})
                    .map(|_| line.clear())
                    .unwrap();
            }
            message = receiver.recv() => {
                let msg = message.unwrap();
                if msg.address != address { write.write_all(format!("{msg}").as_bytes()).await.unwrap() }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .expect("Unable to create TCP server!");
    let (sender, _receiver) = broadcast::channel::<Message>(1024);

    loop {
        listener
            .accept()
            .await
            .map(|(socket, addr)| {
                let sender = sender.clone();
                tokio::spawn(async move {
                    process_single_connection(socket, addr, sender.clone()).await
                });
            })
            .expect("Server unable to handle user connection!");
    }
}
