use openssl::ssl::{Ssl, SslContext, SslMethod, SslStream};
use std::io::prelude::*;
use std::net::TcpStream;
use rori_utils::data::RoriData;
use rustc_serialize::json::decode;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::fs::File;

#[allow(dead_code)]
pub struct RoriClient {
    pub address: String,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
pub struct ConfigServer {
    pub ip: Option<String>,
    pub port: Option<String>,
}

/**
 * RORIClient is used to send data to RORI
 */
#[allow(dead_code)]
impl RoriClient {
    pub fn parse_config(data: String) -> String {
        let params: ConfigServer = decode(&data[..])
            .map_err(|_| {
                Error::new(ErrorKind::InvalidInput,
                           "Failed to decode configuration file.")
            })
            .unwrap();

        format!("{}:{}",
                &params.ip.unwrap_or(String::from("")),
                &params.port.unwrap_or(String::from("")))
    }

    pub fn new<P: AsRef<Path>>(config: P) -> RoriClient {
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let address = RoriClient::parse_config(data);
        if address == ":" {
            error!(target:"RoriClient", "Empty config for the connection to the server");
        }
        RoriClient { address: address }
    }

    pub fn send_to_rori(&mut self,
                        author: &str,
                        content: &str,
                        client: &str,
                        datatype: &str,
                        secret: &str)
                        -> bool {
        let data_to_send = RoriData::new(String::from(author),
                                         String::from(content),
                                         String::from(client),
                                         String::from(datatype),
                                         String::from(secret));


        let context = SslContext::new(SslMethod::Tlsv1).unwrap();
        let ssl = Ssl::new(&context).unwrap();
        let inner = TcpStream::connect(&*self.address).unwrap();
        if let Ok(mut stream) = SslStream::connect(ssl, inner) {
            let _ = stream.write(data_to_send.to_string().as_bytes());
            return true;
        } else {
            error!(target:"RoriClient", "Couldn't connect to RORI at address {}", &*self.address);
            return false;
        }

    }
}
