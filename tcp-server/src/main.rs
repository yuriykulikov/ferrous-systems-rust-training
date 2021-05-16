use std::borrow::BorrowMut;
use std::collections::vec_deque::Iter;
use std::collections::VecDeque;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use redisish::{Command, parse};

trait Mailbox {
    /// TODO Is it fine that mutability is hidden? How can we do it some other way?
    fn append(&self, email: String);
    /// TODO does it make sense to return an abstract iterator and how can this be achieved?
    fn list_emails(&self) -> String;
}

struct VecDequeMailbox {
    data: Mutex<VecDeque<String>>,
}

impl Mailbox for VecDequeMailbox {
    /// TODO is there a way (and does it make sense) to accept &str?
    fn append(&self, email: String) {
        let result = self.data.lock();
        result.unwrap().push_front(email);
    }

    /// Returns a list of emails as a string, separated by `;`
    fn list_emails(&self) -> String {
        self.data.lock()
            .unwrap()
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
fn main() -> io::Result<()> {
    spawn_tcp_listener();
    let client_thread = spawn_monitoring_thread();
    client_thread.join().unwrap();
    Ok(())
}

/// Spawns TcpListener thread, connects to 127.0.0.1:8080 and spawns thread per connection
fn spawn_tcp_listener() {
    thread::spawn(|| {
        let mailbox = Arc::new(VecDequeMailbox::new());
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        for stream in listener.incoming() {
            // TODO is shadowing the variable name idiomatic?
            let mailbox = mailbox.clone();
            // TODO: confirm that dropping the JoinHandle does not do anything
            thread::spawn(|| {
                println!("Server has a new client {:?}", stream);
                handle_client(stream.unwrap(), mailbox);
            });
        }
    });
}

/// TODO how can I pass the mailbox, preferably just mailbox: Mailbox or at least mailbox: Arc<Box<dyn Mailbox>>?
fn handle_client(mut client: TcpStream, mailbox: Arc<Box<VecDequeMailbox>>) -> Result<(), io::Error> {
    loop {
        let mut str = String::new();
        BufReader::new(&client).read_line(&mut str);
        let result = parse(&str);
        match result {
            Ok(Command::Publish(payload)) => {
                println!("Appending email: {}", payload);
                mailbox.append(payload)
            }
            Ok(Command::Retrieve) => {
                client.write_all(mailbox.list_emails().as_ref());
                client.write_all("\n".as_ref());
            }
            Err(err) => {
                println!("Disconnected from {:?}, {}", client, err);
                break;
            }
        }
    }

    Ok(())
}

fn spawn_monitoring_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        // TODO replace with retry
        thread::sleep(Duration::from_millis(1000));
        let mut client = TcpStream::connect("127.0.0.1:8080").unwrap();
        // TODO check diff
        loop {
            client.write_all(Command::Retrieve.as_string().as_ref());
            let mut str = String::new();
            BufReader::new(&client).read_line(&mut str);
            println!("Mailbox content: {}", str.trim_end());
            thread::sleep(Duration::from_millis(5000));
        }
    })
}