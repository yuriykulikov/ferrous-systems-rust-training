use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};

use crate::Model;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// Handles key events. Blocks until a key event or a tick arrives.
/// TODO map errors
/// TODO inject sender
pub fn controller(key_events: &Receiver<Event<KeyEvent>>, model: Arc<Mutex<Model>>) -> bool {
    let mut esc = false;
    match key_events.recv().unwrap() {
        Event::Input(event) => match event.code {
            KeyCode::Esc => {
                esc = true;
            }
            KeyCode::Enter => {
                on_enter(&model);
            }
            KeyCode::Char(input) => {
                model.lock().unwrap().composed_push(input);
            }
            KeyCode::Backspace => {
                model.lock().unwrap().composed_backspace();
            }
            KeyCode::Up => {
                model.lock().unwrap().dec_channel();
            }
            KeyCode::Down => {
                model.lock().unwrap().inc_channel();
            }
            _ => {}
        },
        Event::Tick => {}
    }
    esc
}

/// Sends the email to the server and clears the input
fn on_enter(model: &Arc<Mutex<Model>>) {
    let mut model = model.lock().unwrap();
    let composed_email_content = model.composed();
    let selected_channel_name = model.selected_channel_name();
    let mut client = TcpStream::connect("127.0.0.1:8080").unwrap();

    let message = match selected_channel_name.as_str() {
        "+" => format!(
            "PUBLISH {}: {}\n",
            composed_email_content, "Created the channel"
        ),
        _ => format!(
            "PUBLISH {}: {}\n",
            selected_channel_name, composed_email_content,
        ),
    };

    client.write_all(message.as_ref()).unwrap();

    if selected_channel_name == "+" {
        model.select_channel(composed_email_content.as_str());
    }
    model.composed_clear();
}

/// Emits key events and ticks at least every 200ms
pub fn key_events() -> Receiver<Event<KeyEvent>> {
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }

            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });
    rx
}
