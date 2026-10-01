use crate::layers::network_layer;

mod layers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(err) = network_layer::run().await {
        eprintln!("[ERROR]: {err}");
    };

    Ok(())
}
