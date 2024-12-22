use embedded_recruitment_task::message::{client_message, ServerMessage};
use prost::Message;
use log::info;
use log::error;
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

pub struct Client {
    ip: String,
    port: u32,
    timeout: Duration,
    stream: Option<TcpStream>,
}

impl Client {
    pub fn new(ip: &str, port: u32, timeout_ms: u64) -> Self {
        Client {
            ip: ip.to_string(),
            port,
            timeout: Duration::from_millis(timeout_ms),
            stream: None,
        }
    }

    pub fn connect(&mut self) -> io::Result<()> {
        let address = format!("{}:{}", self.ip, self.port);
        let socket_addrs: Vec<SocketAddr> = address.to_socket_addrs()?.collect();

        if socket_addrs.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid IP or port",
            ));
        }

        let stream = TcpStream::connect_timeout(&socket_addrs[0], self.timeout)?;
        self.stream = Some(stream);
        println!("Connected to the server!");
        Ok(())
    }

    pub fn disconnect(&mut self) -> io::Result<()> {
        if let Some(stream) = self.stream.take() {
            stream.shutdown(std::net::Shutdown::Both)?;
        }
        println!("Disconnected from the server!");
        Ok(())
    }

    pub fn send(&mut self, message: client_message::Message) -> io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            let mut buffer = Vec::new();
    
            // Encode the message (returns `()`)
            message.encode(&mut buffer);
    
            // Write the encoded message to the stream
            stream.write_all(&buffer)?;
            stream.flush()?;

            println!("Sent message: {:?}", message);
            Ok(())
        } else {
              // Error if the stream is not connected
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }
    
    pub fn receive(&mut self) -> io::Result<ServerMessage> {
        if let Some(ref mut stream) = self.stream {
            info!("Receiving message from the server");
            let mut buffer = vec![0u8; 1024]; // Buffer for incoming data
            let bytes_read = stream.read(&mut buffer)?; // Read data from the stream
            if bytes_read == 0 {
                // Error handling for server disconnection
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Server disconnected",
                ));
            }
            info!("Received {} bytes from the server", bytes_read);

             // Decode the message using prost
            ServerMessage::decode(&buffer[..bytes_read]).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to decode ServerMessage: {}", e),
                )
            })
        } else {
            error!("No active connection");
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "No active connection",
            ))
        }
    }
}

