
// sync task exe
pub async fn execute(data: &[u8]) {
    let task = String::from_utf8(data.to_vec()).unwrap();
    println!("Executing task: {}", task);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Sim execute
    println!("SYNC task completed: {}", task);
}
