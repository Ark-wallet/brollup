use crate::{tcp_client::Client, PEER};

// ping
pub async fn command(coordinator: &PEER) {
    match coordinator.ping().await {
        Ok(duration) => println!("{} ms", duration.as_millis()),
        Err(_) => println!("Error pinging."),
    }
}
