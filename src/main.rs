use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::process::Command;

const CHANNELS_FILE: &str = "channels.json";

#[derive(Serialize, Deserialize, Debug)]
struct Channel {
    friendly_name: String,
    handle: String,
}

fn get_choice(range: usize) -> u8 {
    loop {
        let mut input: String = String::new();

        stdin().read_line(&mut input).expect("Failed to read input");

        match input.trim().parse::<u8>() {
            Ok(num) => {
                if (num as usize) <= range {
                    break num;
                } else {
                    println!("A number from the list, please\n");
                    continue;
                }
            }
            Err(_) => {
                println!("Please provide an actual number\n");
                continue;
            }
        }
    }
}

fn main() {
    println!("Who do you want to watch?\n");

    let data: String = read_to_string(CHANNELS_FILE).expect("Could not read file");

    let channels: Vec<Channel> =
        serde_json::from_str(data.as_str()).expect("Could not deserialise JSON");

    println!("Type [0] to input the channel name, or pick one from below\n");

    for (i, channel) in channels.iter().enumerate() {
        println!("[{}] {}", i + 1, channel.friendly_name);
    }

    println!("");

    let choice: u8 = get_choice(channels.len());

    let channel_handle: String = if choice != 0u8 {
        channels[(choice - 1) as usize].handle.to_owned()
    } else {
        println!("\nWhat is the channel name?\n");

        let mut handle: String = String::new();

        stdin()
            .read_line(&mut handle)
            .expect("Failed to read input");

        handle
    };

    print!("\nOpen chat? (Y/n): ");
    stdout().flush().ok().expect("Could not flush stdout");

    let mut chat: String = String::new();

    stdin().read_line(&mut chat).expect("Failed to read input");

    Command::new("powershell")
        .arg("Start-Process")
        .arg("streamlink")
        .arg(format!("twitch.tv/{}", channel_handle.trim()))
        .arg("-WindowStyle")
        .arg("Hidden")
        .output()
        .expect("Failed to open stream");

    if chat.trim().to_lowercase() == "y" || chat.trim().to_lowercase() == "" {
        Command::new("powershell")
            .arg("Start-Process")
            .arg("\"C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Chatterino\"")
            .arg(format!("\"-c {}\"", channel_handle.trim()))
            .output()
            .expect("Failed to open chat");
    }
}
