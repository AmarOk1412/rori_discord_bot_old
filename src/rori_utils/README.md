**Disclaimer: This is a draft version**

# rori_utils

Some Rust source files designed to help programmers to create entry or endpoint for _[RORI](https://github.com/AmarOk1412/RORI)_.

## Create an EntryPoint

An EntryPoint is a point which get data and send it to RORI. To create an EntryPoint you can use `rori_utils::client::RoriClient`:

```rust
pub struct RoriClient {
    pub address: String, // address = "ip:port"
}

impl RoriClient {
  /**
   * To create a RoriClient from a file which contains:
   * {
   *   "ip":"ip of the rori_server you want",
   *   "port":"port of the rori_server you want"
   * }
   */
  pub fn new<P: AsRef<Path>>(config: P) -> RoriClient { /*...*/ }

  /**
   * Send data to the rori_server
   */
  pub fn send_to_rori(&mut self,
                        author: &str,
                        content: &str,
                        client: &str,
                        datatype: &str,
                        secret: &str)
                        -> bool { /*...*/ }
}
```

And that's all you need. If you want, you can see an example [here](https://github.com/AmarOk1412/irc_entry_module/blob/master/src/main.rs)

## Create an Endpoint

The endpoint process commands from RORI. You can discover how to configure an endpoint [here](https://github.com/AmarOk1412/RORI/wiki/Create-your-endpoint). In `rori_modules`, you can use `rori_utils::RoriEndpoint` to help you to develop your own endpoint. Your endpoint will implement this trait:

```rust
pub trait Endpoint {
    // Start a TLS server and process incoming data
    fn start(&self);
    // Return true if the data is an authorized data to process
    fn is_authorized(&self, data: RoriData) -> bool;
    // Register the endpoint
    fn register(&mut self);
}
```

By the way, you can directly create a **RoriEndpoint** from a json config file (see the wiki for details) with:

```rust
impl RoriEndpoint {
  pub fn new<P: AsRef<Path>>(config: P) -> RoriEndpoint { /*...*/ }
}
```

This is a complete example of a simple endpoint (from [RORI Discord Bot](https://github.com/AmarOk1412/rori_discord_bot)):

```rust
use openssl::ssl::{SslContext, SslMethod, SslStream, SSL_VERIFY_NONE};
use openssl::x509::X509FileType::PEM;
use rori_utils::data::RoriData;
use rori_utils::endpoint::{Endpoint, Client, RoriEndpoint};
use std::path::Path;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub struct DiscordEndpoint {
    endpoint: RoriEndpoint,
    incoming_data: Arc<Mutex<Vec<String>>>,
}

impl Endpoint for DiscordEndpoint {
    fn start(&self) {
        let vec = self.incoming_data.clone();
        let listener = TcpListener::bind(&*self.endpoint.address).unwrap();
        let mut ssl_context = SslContext::new(SslMethod::Tlsv1).unwrap();
        match ssl_context.set_certificate_file(&*self.endpoint.cert.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Certificate set"),
            Err(_) => error!(target:"Server", "Can't set certificate file"),
        };
        ssl_context.set_verify(SSL_VERIFY_NONE, None);
        match ssl_context.set_private_key_file(&*self.endpoint.key.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Private key set"),
            Err(_) => error!(target:"Server", "Can't set private key"),
        };
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let ssl_stream = SslStream::accept(&ssl_context, stream.try_clone().unwrap());
                    let ssl_ok = match ssl_stream {
                        Ok(_) => true,
                        Err(_) => false,
                    };
                    if ssl_ok {
                        let ssl_stream = ssl_stream.unwrap();
                        let mut client = Client::new(ssl_stream.try_clone().unwrap());
                        let content = client.read();
                        info!(target:"endpoint", "Received:{}", &content);
                        let end = content.find(0u8 as char);
                        let (content, _) = content.split_at(end.unwrap_or(content.len()));
                        let data_to_process = RoriData::from_json(String::from(content));
                        let data_authorized = self.is_authorized(data_to_process.clone());
                        if data_authorized {
                            if data_to_process.datatype == "text" {
                                vec.lock().unwrap().push(data_to_process.content);
                            }
                        } else {
                            error!(target:"Server", "Stream not authorized! Don't process.");
                        }
                    } else {
                        error!(target:"Server", "Can't create SslStream");
                    }
                }
                Err(e) => {
                    error!(target:"endpoint", "{}", e);
                }
            };
        }
        drop(listener);
    }

    fn is_authorized(&self, data: RoriData) -> bool {
        self.endpoint.is_authorized(data)
    }

    fn register(&mut self) {
        self.endpoint.register()
    }
}

impl DiscordEndpoint {
    pub fn new<P: AsRef<Path>>(config: P,
                               incoming_data: Arc<Mutex<Vec<String>>>)
                               -> DiscordEndpoint {
        DiscordEndpoint {
            endpoint: RoriEndpoint::new(config),
            incoming_data: incoming_data,
        }
    }

    pub fn is_registered(&self) -> bool {
        self.endpoint.is_registered
    }
}
```

In this example, you just have to implement the start function which launch a Tls Server and process commands.

## TODO

Move in a library?
