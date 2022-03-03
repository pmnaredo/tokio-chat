use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    // A TCP socket server, listening for connections.
    //
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening on port 8080...");

    // Create a bounded, multi-producer, multi-consumer channel where each sent
    // value is broadcasted to all active receivers
    //
    let (tx, _rx) = broadcast::channel(10);

    loop {
        // Accept a new incoming connection from this listener.
        //
        let (mut socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // Spawn a new asynchronous task for handling the new client.
        //
        tokio::spawn(async move {
            // Split the TcpStream into a read half and a write half, which can be used
            // to read and write the stream concurrently.
            //
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                // Wait on multiple concurrent branches, returning when the first branch
                // completes, cancelling the remaining branches.
                //
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        // Message is sent to the client
                        //
                        if addr != other_addr {
                            writer.write_all(format!(">>> {}", msg).as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
