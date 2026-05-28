use oxidite_core::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello from Oxidite!");
    println!("This is a standalone Rust file running via oxidite-cli");
    println!();
    println!("Current time: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
    
    Ok(())
}
