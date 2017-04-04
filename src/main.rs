extern crate crypto;
extern crate discord;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate rustc_serialize;
extern crate regex;

mod rori_utils;
mod endpoint;

use discord::{Discord, State};
use discord::model::Event;
use discord::model::ChannelId;
use endpoint::DiscordEndpoint;
use rori_utils::client::RoriClient;
use rori_utils::endpoint::Endpoint;
use rustc_serialize::json::decode;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct BotDetails {
    token: String,
    channel: String,
    secret: String,
    name: String,
    botname: String,
}

/**
 * DiscordBot
 */
struct DiscordBot {
    token: String,
    channel: String,
    secret: String,
    name: String,
    botname: String,
}

impl DiscordBot {
    fn new<P: AsRef<Path>>(config: P) -> DiscordBot {
        info!(target: "DiscordBot", "init");
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let details: BotDetails = decode(&data[..]).unwrap();
        DiscordBot {
            token: details.token,
            channel: details.channel,
            secret: details.secret,
            name: details.name,
            botname: details.botname,
        }
    }

    /**
     * For each message received by the bot. Make actions.
     * @param: client: where we send messages from Discord
     * @param: incoming: what the bot says on Discord
     */
    fn process_msg(&self, client: &mut RoriClient, incoming: Arc<Mutex<Vec<String>>>) {
        info!(target: "DiscordBot", "process_msg");

        // Log in to Discord using a bot token from the environment
        let discord = Discord::from_bot_token(&*self.token).expect("login failed");

        // Establish and use a websocket connection
        let (mut connection, ready) = discord.connect().expect("connect failed");
        let id_bot = ready.user.id.clone();
        let mut state = State::new(ready);
        connection.sync_calls(&state.all_private_channels());


        let discord_cloned = Discord::from_bot_token(&*self.token).expect("login failed");
        // let server_id = discord.get_servers().unwrap()[0].id;
        let id_to_process = self.channel.parse::<u64>().unwrap();
        let channel_id = ChannelId(id_to_process);
        let channel_cloned = channel_id.clone();
        thread::spawn(move || {
            loop {
                if incoming.lock().unwrap().len() != 0 {
                    match incoming.lock().unwrap().pop() {
                        Some(s) => {
                            info!(target:"DiscordBot", "write: {}", &s);
                            let _ = discord_cloned.send_message(channel_cloned, &s, "", false);
                        }
                        None => {}
                    }
                }
            }
        });

        loop {
            let event = match connection.recv_event() {
                Ok(event) => event,
                Err(err) => {
                    println!("[Warning] Receive error: {:?}", err);
                    if let discord::Error::WebSocket(..) = err {
                        // Handle the websocket connection being dropped
                        let (new_connection, ready) = discord.connect().expect("connect failed");
                        connection = new_connection;
                        state = State::new(ready);
                        println!("[Ready] Reconnected successfully.");
                    }
                    if let discord::Error::Closed(..) = err {
                        break;
                    }
                    continue;
                }
            };
            state.update(&event);


            match event {
                Event::MessageCreate(message) => {
                    info!(target: "DiscordBot", "{}, {} says: {} in {:?}",&message.author.id, &message.author.name, &message.content, message.channel_id);
                    if message.content == "!voice" {
                        info!("Join voice channel");
                        let vchan = state.find_voice_user(message.author.id);
                        let output = if let Some((server_id, channel_id)) = vchan {
                            let voice = connection.voice(server_id);
                            voice.set_deaf(true);
                            voice.connect(channel_id);
                            String::new()
                        } else {
                            "You must be in a voice channel to voice".to_owned()
                        };
                        if !output.is_empty() {
                            println!("{:?}", output);
                        }
                    }
                    if &message.author.name != &*self.botname && message.channel_id == channel_id {
                        client.send_to_rori(&message.author.name,
                                            &message.content,
                                            &self.name,
                                            "text",
                                            &self.secret);
                    }
                    if &message.author.name == &*self.botname {
                        let vchan = state.find_voice_user(id_bot);
                        let mut child = Command::new("python3")
                            .arg("scripts/mimic.py")
                            .arg(&message.content)
                            .spawn()
                            .expect("mimic.py command failed to start");
                        child.wait()
                            .expect("failed to wait on child");
                        let output = if let Some((server_id, channel_id)) = vchan {
                            match discord::voice::open_ffmpeg_stream("output.wav") {
                                Ok(stream) => {
                                    let voice = connection.voice(server_id);
                                    voice.set_deaf(true);
                                    voice.connect(channel_id);
                                    voice.play(stream);
                                    String::new()
                                }
                                Err(error) => format!("Error: {}", error),
                            }
                        } else {
                            "You must be in a voice channel to DJ".to_owned()
                        };
                        if !output.is_empty() {
                            println!("{:?}", output);
                            // warn(discord.send_message(message.channel_id, &output, "", false));
                        }
                    }
                }
                Event::VoiceStateUpdate(server_id, _) => {
                    // If someone moves/hangs up, and we are in a voice channel,
                    if let Some(cur_channel) = connection.voice(server_id).current_channel() {
                        // and our current voice channel is empty, disconnect from voice
                        match server_id {
                            Some(server_id) => {
                                if let Some(srv) = state.servers()
                                    .iter()
                                    .find(|srv| srv.id == server_id) {
                                    if srv.voice_states
                                        .iter()
                                        .filter(|vs| vs.channel_id == Some(cur_channel))
                                        .count() <= 1 {
                                        connection.voice(Some(server_id)).disconnect();
                                    }
                                }
                            }
                            None => {
                                if let Some(call) = state.calls().get(&cur_channel) {
                                    if call.voice_states.len() <= 1 {
                                        connection.voice(server_id).disconnect();
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {} // discard other events
            }
        }
    }
}


#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
pub struct Secret {
    pub secret: Option<String>,
}

fn main() {
    // Init logging
    env_logger::init().unwrap();

    // will contains messages from RORI
    let incoming = Arc::new(Mutex::new(Vec::new()));
    let incoming_cloned = incoming.clone();
    let child_endpoint = thread::spawn(move || {
        let mut endpoint = DiscordEndpoint::new("config_endpoint.json", incoming);
        endpoint.register();
        if endpoint.is_registered() {
            endpoint.start();
        } else {
            error!(target: "endpoint", "endpoint is not registered");
        }
    });

    let mut client = RoriClient::new("config_server.json");
    let rori = DiscordBot::new("config_endpoint.json");
    rori.process_msg(&mut client, incoming_cloned);
    let _ = child_endpoint.join();
}
