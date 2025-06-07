use std::env;
use std::time::Instant;

use anyhow::Result;
use serenity::prelude::*;
use serenity::model::channel::ChannelId;
use tracing::{info, error};

pub struct DiscordClient {
    client: Client,
    channel_id: ChannelId,
}

impl DiscordClient {
    pub async fn new() -> Result<Self> {
        let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
        
        // Validate token format
        if let Err(e) = serenity::utils::token::validate(&token) {
            return Err(anyhow::anyhow!("Invalid Discord token format: {}", e));
        }

        let channel_id = env::var("DISCORD_CHANNEL_ID")
            .expect("Expected DISCORD_CHANNEL_ID in environment")
            .parse::<u64>()?;

        let intents = GatewayIntents::GUILD_MESSAGES;

        let client = Client::builder(&token, intents)
            .await
            .expect("Error creating Discord client");

        Ok(Self {
            client,
            channel_id: ChannelId(channel_id),
        })
    }

    pub async fn send_circuit_event(&self, event: &crate::gpio::CircuitEvent) -> Result<()> {
        let start = Instant::now();
        info!("Sending Discord message for circuit event: {:?}", event);

        let message = format!("Circuit state changed: {:?}", event);
        
        if let Err(why) = self.channel_id.say(&self.client.http, message).await {
            error!("Error sending Discord message: {:?}", why);
            return Err(anyhow::anyhow!("Failed to send Discord message: {}", why));
        }

        let duration = start.elapsed();
        info!("Discord message sent successfully in {:?}", duration);
        Ok(())
    }
} 