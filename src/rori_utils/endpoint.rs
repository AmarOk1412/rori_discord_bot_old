use crypto::digest::Digest;
use crypto::sha2::Sha256;
use openssl::ssl::SslStream;
use rori_utils::data::RoriData;
use rori_utils::client::{RoriClient, ConfigServer};
use rustc_serialize::json::decode;
use std::path::Path;
use std::net::TcpStream;
use std::str::from_utf8;
use std::io::prelude::*;
use std::fs::File;

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
pub struct AuthorizedUser {
    pub name: Option<String>,
    pub secret: Option<String>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct RoriServer {
    pub rori_ip: Option<String>,
    pub rori_port: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub secret: Option<String>,
    pub authorize: Vec<AuthorizedUser>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct EndpointDetails {
    owner: Option<String>,
    name: Option<String>,
    compatible_types: Option<String>,
}

#[allow(dead_code)]
/**
 * RoriEndpoint is used to handle data from RORI
 */
pub struct RoriEndpoint {
    pub address: String,
    pub rori_address: String,
    pub is_registered: bool,
    pub owner: String,
    pub name: String,
    pub compatible_types: String,
    pub cert: String,
    pub key: String,
    pub secret: String,
    pub authorize: Vec<AuthorizedUser>,
}

#[allow(dead_code)]
pub struct Client {
    stream: SslStream<TcpStream>,
}

#[allow(dead_code)]
impl Client {
    pub fn new(stream: SslStream<TcpStream>) -> Client {
        return Client { stream: stream };
    }

    pub fn read(&mut self) -> String {
        let mut result = String::from("");
        let mut buffer = [0u8; 512];
        loop {
            let usize = self.stream.read(&mut buffer).unwrap();
            if usize == 0 {
                break;
            }
            let msg = from_utf8(&buffer).unwrap();
            result.push_str(msg);
        }
        result
    }
}

pub trait Endpoint {
    fn start(&self);
    fn is_authorized(&self, data: RoriData) -> bool;
    fn register(&mut self);
}

impl Endpoint for RoriEndpoint {
    fn start(&self) {
        info!("Not implemented");
    }

    /**
     * Get if a data come from an authorized Client
     * @param data: the data to process
     * @return true if the data is authorized, false else
     */
    fn is_authorized(&self, data: RoriData) -> bool {
        let mut hasher = Sha256::new();
        hasher.input_str(&*data.secret);
        let secret = hasher.result_str();
        for client in self.authorize.clone() {
            if client.name.unwrap().to_lowercase() == data.client.to_lowercase() &&
               secret.to_lowercase() == client.secret.unwrap().to_lowercase() {
                return true;
            }
        }
        false
    }

    /**
     * Register an endpoint
     */
    fn register(&mut self) {
        info!(target:"endpoint", "try to register endpoint");
        let rori_address = self.rori_address.clone();
        let address = self.address.clone();
        let mut client = RoriClient { address: rori_address };
        let mut content = String::from(address);
        content.push_str("|");
        content.push_str(&*self.compatible_types);
        self.is_registered =
            client.send_to_rori(&self.owner, &*content, &self.name, "register", &self.secret);
    }
}

impl RoriEndpoint {
    fn parse_config_server(data: String) -> String {
        let params: ConfigServer = decode(&data[..]).unwrap();
        format!("{}:{}",
                    &params.ip.unwrap_or(String::from("")),
                    &params.port.unwrap_or(String::from("")))
    }

    pub fn new<P: AsRef<Path>>(config: P) -> RoriEndpoint {
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let address = RoriEndpoint::parse_config_server(data.clone());
        let params: RoriServer = decode(&data[..]).unwrap();
        let rori_address = format!("{}:{}",
                                       &params.rori_ip.unwrap_or(String::from("")),
                                       &params.rori_port.unwrap_or(String::from("")));
        let details: EndpointDetails = decode(&data[..]).unwrap();
        if address == ":" || rori_address == ":" {
            error!(target:"endpoint", "Empty config for the connection to the server");
        }
        RoriEndpoint {
            address: address,
            rori_address: rori_address,
            is_registered: false,
            owner: details.owner.unwrap_or(String::from("")),
            name: details.name.unwrap_or(String::from("")),
            compatible_types: details.compatible_types.unwrap_or(String::from("")),
            cert: params.cert.unwrap_or(String::from("")),
            key: params.key.unwrap_or(String::from("")),
            secret: params.secret.unwrap_or(String::from("")),
            authorize: params.authorize,
        }
    }
}
