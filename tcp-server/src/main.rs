use std::borrow::BorrowMut;
use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::io;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;

use redisish::{parse, Command};

struct VecDequeMailbox {
    data: Mutex<VecDeque<String>>,
}

impl VecDequeMailbox {
    async fn append(&self, email: &str) {
        let mut result = self.data.lock().await;
        result.push_front(email.to_owned());
    }

    /// Returns a list of emails as a string, separated by `;`
    async fn list_emails(&self) -> String {
        self.data
            .lock()
            .await
            .iter()
            .fold(String::new(), |acc, next| acc + next + ";")
    }
}

impl VecDequeMailbox {
    // TODO how to return Box<dyn Mailbox>?
    fn new() -> Box<VecDequeMailbox> {
        return Box::new(VecDequeMailbox {
            data: Mutex::new(VecDeque::new()),
        });
    }
}

/// General questions:
/// * how to work with `dyn Mailbox ` in multithreaded environment?
#[tokio::main]
async fn main() -> io::Result<()> {
    tokio::join!(spawn_monitoring_thread(), spawn_tcp_listener(),);
    Ok(())
}

/// Spawns TcpListener thread, connects to 127.0.0.1:8080 and spawns thread per connection
fn spawn_tcp_listener() -> JoinHandle<()> {
    return tokio::spawn(async {
        let mailbox = Arc::new(VecDequeMailbox::new());
        let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

        loop {
            let next = listener.accept().await.unwrap();
            let mailbox = mailbox.clone();
            tokio::spawn(async {
                handle_client(BufReader::new(next.0), mailbox).await;
            });
        }
    });
}

/// TODO how can I pass the mailbox, preferably just mailbox: Mailbox or at least mailbox: Arc<Box<dyn Mailbox>>?
async fn handle_client(
    mut tcp_stream: BufReader<TcpStream>,
    mailbox: Arc<Box<VecDequeMailbox>>,
) -> Result<(), io::Error> {
    loop {
        let mut str = String::new();
        let ret = tcp_stream.read_line(&mut str).await;
        if let Ok(0) = ret {
            break;
        }
        let result = parse(&str);
        match result {
            Ok(Command::Publish(payload)) => {
                println!("Appending email: {}", payload);
                mailbox.append(payload.as_ref()).await;
            }
            Ok(Command::Retrieve) => {
                tcp_stream
                    .write_all(mailbox.list_emails().await.as_ref())
                    .await;
                tcp_stream.write_all("\n".as_ref()).await;
            }
            Err(err) => {
                println!("Client error: {}", err);
                break;
            }
        }
    }

    Ok(())
}

fn spawn_monitoring_thread() -> JoinHandle<()> {
    tokio::spawn(async {
        // TODO replace with retry
        time::sleep(Duration::from_millis(1000)).await;
        let mut client = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let mut client = BufReader::new(client);
        loop {
            client
                .write_all(Command::Retrieve.as_string().as_ref())
                .await;
            let mut str = String::new();
            client.read_line(&mut str).await;
            println!("Mailbox content: {}", str.trim_end());
            time::sleep(Duration::from_millis(5000)).await;
        }
    })
}
