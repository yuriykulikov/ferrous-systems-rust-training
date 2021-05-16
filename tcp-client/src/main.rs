use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream};

use shrust::{Shell, ShellIO};

use redisish::{Command};

fn main() {
    let mut shell = Shell::new(());
    println!("Type help to list available commands!");

    shell.new_command_noargs("list", "List all emails", |io, _| {
        let mut client = TcpStream::connect("127.0.0.1:8080")?;
        client.write_all(Command::Retrieve.as_string().as_ref())?;
        let mut str = String::new();
        BufReader::new(&client).read_line(&mut str).unwrap();
        str.trim_end().split(';').for_each(|mail| {
            writeln!(io, "{}", mail).unwrap();
        });
        Ok(())
    });

    shell.new_command("publish", "Publish a message", 1, |io, _, s| {
        let mut client = TcpStream::connect("127.0.0.1:8080").unwrap();
        writeln!(io, "Publishing {}", s.join(" "))?;
        client.write_all(format!("PUBLISH {}\n", s.join(" ")).as_ref())?;
        Ok(())
    });

    shell.run_loop(&mut ShellIO::default());
}
