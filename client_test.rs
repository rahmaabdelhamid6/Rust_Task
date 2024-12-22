use embedded_recruitment_task::{
    message::{client_message, server_message, AddRequest, EchoMessage},
    server::Server,
};
use std::{
    sync::{Arc,Mutex},
    thread::{self, JoinHandle},  // Added Mutex for thread-safe client handling
    time::Duration,
};
mod client;

fn setup_server_thread(server: Arc<Server>) -> JoinHandle<()> {
    thread::spawn(move || {
        server.run().expect("Server encountered an error");
    })
}

// Modified to accept a dynamic port number
fn create_server(port: u32) -> Arc<Server> {
    Arc::new(Server::new(&format!("localhost:{}", port)).expect("Failed to start server"))
}

// New helper function to get a free port for testing
fn get_free_port() -> u16 {
    // Use std::net to bind to port 0, which will allocate a free port
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to free port")
        .local_addr()
        .unwrap()
        .port()
}


#[test]
fn test_client_connection() {
    let port = get_free_port() as u32; // Dynamically allocate a free port
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

     // Wait for the server to fully start
    thread::sleep(Duration::from_secs(1)); // Adjust this time if necessary

    let mut client = client::Client::new("localhost", port, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    
    server.stop(); // Ensure the server stops after the test
    handle.join().expect("Server thread failed to join");
}

#[test]
fn test_client_echo_message() {
    let port = get_free_port() as u32;  // Dynamically allocate a free port
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", port, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut echo_message = EchoMessage::default();
    echo_message.content = "Hello, World!".to_string();
    let message = client_message::Message::EchoMessage(echo_message.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(response.is_ok(), "Failed to receive response for EchoMessage");

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(
                echo.content, echo_message.content,
                "Echoed message content does not match"
            );
        }
        _ => panic!("Expected EchoMessage, but received a different message"),
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_multiple_echo_messages() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", port, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message);

        assert!(client.send(message).is_ok(), "Failed to send message");

        let response = client.receive();
        assert!(response.is_ok(), "Failed to receive response for EchoMessage");

        match response.unwrap().message {
            Some(server_message::Message::EchoMessage(echo)) => {
                assert_eq!(
                    echo.content, message_content,
                    "Echoed message content does not match"
                );
            }
            _ => panic!("Expected EchoMessage, but received a different message"),
        }
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_multiple_clients() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let mut clients = vec![
        client::Client::new("localhost", port, 1000),
        client::Client::new("localhost", port, 1000),
        client::Client::new("localhost", port, 1000),
    ];

    for client in clients.iter_mut() {
        assert!(client.connect().is_ok(), "Failed to connect to the server");
    }

    let messages = vec![
        "Hello, World!".to_string(),
        "How are you?".to_string(),
        "Goodbye!".to_string(),
    ];

    for message_content in messages {
        let mut echo_message = EchoMessage::default();
        echo_message.content = message_content.clone();
        let message = client_message::Message::EchoMessage(echo_message.clone());

        for client in clients.iter_mut() {
            assert!(client.send(message.clone()).is_ok(), "Failed to send message");

            let response = client.receive();
            assert!(response.is_ok(), "Failed to receive response for EchoMessage");

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content, message_content,
                        "Echoed message content does not match"
                    );
                }
                _ => panic!("Expected EchoMessage, but received a different message"),
            }
        }
    }

    for client in clients.iter_mut() {
        assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");
    }

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

#[test]
fn test_client_add_request() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", port, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let mut add_request = AddRequest::default();
    add_request.a = 10;
    add_request.b = 20;
    let message = client_message::Message::AddRequest(add_request.clone());

    assert!(client.send(message).is_ok(), "Failed to send message");

    let response = client.receive();
    assert!(response.is_ok(), "Failed to receive response for AddRequest");

    match response.unwrap().message {
        Some(server_message::Message::AddResponse(add_response)) => {
            assert_eq!(
                add_response.result,
                add_request.a + add_request.b,
                "AddResponse result does not match"
            );
        }
        _ => panic!("Expected AddResponse, but received a different message"),
    }

    assert!(client.disconnect().is_ok(), "Failed to disconnect from the server");

    server.stop();
    assert!(handle.join().is_ok(), "Server thread panicked or failed to join");
}

/// Edge Case: Test invalid server address
#[test]
fn test_invalid_server_address() {
    let port = get_free_port() as u32;
    let mut client: client::Client = client::Client::new("invalid_host", port, 1000);
    assert!(client.connect().is_err(), "Client should fail to connect to an invalid host");
}

/// Edge Case: Test handling empty messages
#[test]
fn test_empty_message() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let mut client = client::Client::new("localhost", port, 1000);
    assert!(client.connect().is_ok(), "Failed to connect to the server");

    let empty_message = client_message::Message::EchoMessage(EchoMessage { content: "".into() });
    assert!(client.send(empty_message).is_ok(), "Failed to send an empty message");

    let response = client.receive();
    assert!(response.is_ok(), "Failed to receive response for empty message");

    match response.unwrap().message {
        Some(server_message::Message::EchoMessage(echo)) => {
            assert_eq!(echo.content, "", "Empty message content does not match");
        }
        _ => panic!("Expected EchoMessage, received something else"),
    }

    client.disconnect().unwrap();
    server.stop();
    handle.join().expect("Server thread failed to join");
}


#[test]
fn test_server_high_load() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let num_clients = 50; // Simulate high server load with 50 clients
    let mut handles = vec![];

    for _ in 0..num_clients {
        let port = port;
        handles.push(thread::spawn(move || {
            let mut client = client::Client::new("localhost", port, 1000);
            assert!(client.connect().is_ok(), "Failed to connect to server");

            for j in 0..10 {
                let message = client_message::Message::EchoMessage(EchoMessage {
                    content: format!("Load test message {}", j),
                });
                assert!(client.send(message).is_ok(), "Failed to send message");
                assert!(client.receive().is_ok(), "Failed to receive response");
            }

            client.disconnect().unwrap();
        }));
    }

    for handle in handles {
        handle.join().expect("Client thread panicked");
    }

    server.stop();
    handle.join().expect("Server thread failed to join");
}
/// Concurrency Test: Multiple clients simultaneously send and receive messages
#[test]
fn test_concurrent_clients() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let num_clients = 10;
    let mut handles = vec![];

    for i in 0..num_clients {
        let port = port;
        handles.push(thread::spawn(move || {
            let mut client = client::Client::new("localhost", port, 1000);
            assert!(client.connect().is_ok(), "Client failed to connect");

            let message = client_message::Message::EchoMessage(EchoMessage {
                content: format!("Hello from client {}", i),
            });

            assert!(client.send(message.clone()).is_ok(), "Client failed to send message");

            let response = client.receive();
            assert!(response.is_ok(), "Client failed to receive response");

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content,
                        format!("Hello from client {}", i),
                        "Echoed content mismatch"
                    );
                }
                _ => panic!("Expected EchoMessage, received something else"),
            }

            assert!(client.disconnect().is_ok(), "Client failed to disconnect");
        }));
    }

    for handle in handles {
        handle.join().expect("Client thread panicked");
    }

    server.stop();
    handle.join().expect("Server thread failed to join");
}

/// Concurrency Test: Single client sends multiple requests simultaneously
#[test]
// Mutex used for thread-safe client handling in single client multiple requests
fn test_single_client_multiple_requests() {
    let port = get_free_port() as u32;
    let server = create_server(port);
    let handle = setup_server_thread(server.clone());

    let client = Arc::new(Mutex::new(client::Client::new("localhost", port, 1000)));
    {
        let mut client_guard = client.lock().unwrap();
        assert!(client_guard.connect().is_ok(), "Failed to connect to the server");
    }

    let num_requests = 5;
    let mut handles = vec![];

    for i in 0..num_requests {
        let client_clone = Arc::clone(&client);
        handles.push(std::thread::spawn(move || {
            let message = client_message::Message::EchoMessage(EchoMessage {
                content: format!("Request {}", i),
            });

            let mut client_guard = client_clone.lock().unwrap();
            assert!(client_guard.send(message.clone()).is_ok(), "Failed to send message");

            let response = client_guard.receive();
            assert!(response.is_ok(), "Failed to receive response");

            match response.unwrap().message {
                Some(server_message::Message::EchoMessage(echo)) => {
                    assert_eq!(
                        echo.content,
                        format!("Request {}", i),
                        "Echoed content mismatch"
                    );
                }
                _ => panic!("Expected EchoMessage, received something else"),
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Request thread panicked");
    }

    {
        let mut client_guard = client.lock().unwrap();
        assert!(client_guard.disconnect().is_ok(), "Failed to disconnect from the server");
    }

    server.stop();
    handle.join().expect("Server thread failed to join");
}

