use anyhow::Ok;
use redis::Client;

async fn generateSend(client: &Client) -> anyhow::Result<()> {
    let conn=client.get_connection_manager().await?;
    
    Ok(())
}
