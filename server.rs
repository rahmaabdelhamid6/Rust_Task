use crate::message::{ClientMessage, client_message, ServerMessage, server_message, AddResponse};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];

        loop {
            match self.stream.read(&mut buffer) {  // Read from the client stream in a loop.
                Ok(0) => {
                    info!("Client disconnected.");
                    return Ok(()); // If no data is read, client is disconnected.
                }
                Ok(bytes_read) => {
                    let received_data = &buffer[..bytes_read];  // Slice the buffer to received data.

                    match ClientMessage::decode(received_data) {  // Decode the incoming message.
                        Ok(client_msg) => {
                            info!("Decoded client message: {:?}", client_msg);  // Log the decoded message.

                            if let Some(message) = client_msg.message {  // Check if there is a message.
                                let response = match message {  // Match on the message type.
                                    client_message::Message::EchoMessage(echo_message) => {
                                        info!("Received EchoMessage: {}", echo_message.content);  // Log EchoMessage content.
                                        server_message::Message::EchoMessage(echo_message)  // Respond with EchoMessage.
                                    }
                                    client_message::Message::AddRequest(add_request) => {
                                        let result = add_request.a + add_request.b;  // Perform addition for AddRequest.
                                        server_message::Message::AddResponse(AddResponse { result })  // Respond with AddResponse.
                                    }
                                };

                                let mut response_buffer = Vec::new();  // Prepare a buffer to encode the response.
                                let server_msg = ServerMessage {
                                    message: Some(response),  // Set the response in the server message.
                                };
                                if let Err(e) = server_msg.encode(&mut response_buffer) {  // Encode the response.
                                    error!("Failed to encode response: {}", e);  // Log encoding failure.
                                    continue;  // Skip the current iteration if encoding fails.
                                }
                                if let Err(e) = self.stream.write_all(&response_buffer) {  // Send the encoded response.
                                    error!("Failed to send response: {}", e);  // Log if sending fails.
                                    break;
                                }
                            } else {
                                warn!("ClientMessage contained no message");  // Warn if no message is present.
                            }
                        }
                        Err(e) => {
                            warn!("Failed to decode message: {}", e);  // Warn if message decoding fails.
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break;  // Exit the loop if there is no data to read.
                }
                Err(e) => {
                    error!("Error reading from client: {}", e);  // Log any read error from the client.
                    return Err(e);  // Return the error if reading fails.
                }
            }
        }

        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;  // Bind the server to the provided address.
        let is_running = Arc::new(AtomicBool::new(false));  // Atomic boolean to track if the server is running.
        Ok(Server {
            listener,
            is_running,
        })
    }

    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst);  // Mark server as running.
        info!("Server running on {}", self.listener.local_addr()?);  // Log the server address.
        self.listener.set_nonblocking(true)?;  // Set listener to non-blocking mode.

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    let is_running = self.is_running.clone();  // Clone the running flag for the thread.
                    thread::spawn(move || {  // Spawn a new thread to handle the client.
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {  // While the server is running.
                            if let Err(e) = client.handle() {  // Handle client communication.
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }
                        info!("Client {} disconnected.", addr);  // Log client disconnection.
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));  // Sleep briefly if there are no connections.
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);  // Log error if accepting connection fails.
                }
            }
        }

        info!("Server stopped.");  // Log when server stops.
        Ok(())
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);  // Set the server running flag to false.
        info!("Server stopping.");  // Log when the server is stopped.
    }
}
