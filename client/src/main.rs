use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

async fn get_user_name() -> String {
    let mut user_name = String::new();
    println!("Enter your name :");
    std::io::stdin().read_line(&mut user_name).unwrap();
    user_name
}

#[tokio::main]
async fn main() {
    let user_name_join = tokio::spawn(get_user_name());
    let connection_join = TcpStream::connect("localhost:8080");

    let (user_name, mut connection) = (
        user_name_join.await.unwrap(),
        connection_join.await.unwrap(),
    );
    let (read, mut write) = connection.split();

    let mut chat_messages_reader = BufReader::new(read);
    let mut chat_message = String::new();

    let mut user_input_reader = BufReader::new(tokio::io::stdin());
    let mut user_message = String::new();

    write.write_all(user_name.as_bytes()).await.unwrap();

    loop {
        tokio::select! {
            _ = chat_messages_reader.read_line(&mut chat_message) => {
                print!("{}", chat_message);
                chat_message.clear();
            }
            _ = user_input_reader.read_line(&mut user_message) => {
                write.write_all(user_message.as_bytes()).await.map(|_| user_message.clear()).unwrap();
            }
        }
    }
}
