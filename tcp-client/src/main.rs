use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use controller::*;
use redisish::Command;
use view::draw_tui;

use crate::model::Model;

mod controller;
mod model;
mod view;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let model: Arc<Mutex<Model>> = Arc::new(Mutex::new(Model::default()));

    spawn_tcp_thread(model.clone());

    let key_events = key_events();
    loop {
        draw_tui(&mut terminal, model.clone())?;
        let esc = controller(&key_events, model.clone());
        if esc {
            disable_raw_mode()?;
            terminal.show_cursor()?;
            break;
        }
    }
    Ok(())
}

/// Spawns a thread this modifies the model when new emails arrive
fn spawn_tcp_thread(model: Arc<Mutex<Model>>) {
    thread::spawn(move || {
        loop {
            match TcpStream::connect("127.0.0.1:8080") {
                Ok(mut client) => {
                    loop {
                        // email1;email2;email3\n
                        let mut str = String::new();
                        let result = client.write_all(Command::Retrieve.as_string().as_ref());
                        if result.is_ok() {
                            BufReader::new(&client).read_line(&mut str).unwrap();
                            let mut emails: Vec<String> =
                                str.trim_end().split(';').map(|it| it.to_owned()).collect();
                            emails.reverse();
                            model.lock().unwrap().replace_emails(emails);
                        } else {
                            break;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                }
                Err(e) => {
                    model.lock().unwrap().replace_emails(vec![
                        format!("Error occurred: {}", e),
                        "Have you started the tcp-server?".to_string(),
                    ]);
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    });
}
