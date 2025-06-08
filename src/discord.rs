use std::env;
use std::time::Instant;

use anyhow::Result;
use serenity::all::{prelude::*, Color, CreateEmbed, CreateMessage};
use serenity::model::id::ChannelId;
use tracing::{info, error};

const COLOR_OPEN: Color = Color::new(0x00FF00);    // Green for open
const COLOR_CLOSED: Color = Color::new(0xFF0000);  // Red for closed

#[derive(Debug)]
pub enum SpaceEvent {
    Open,
    Closed,
    Initializing,
}

pub struct DiscordClient {
    client: Client,
}

impl DiscordClient {
    pub async fn new() -> Result<Self> {
        let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
        
        if let Err(e) = serenity::utils::token::validate(&token) {
            return Err(anyhow::anyhow!("Invalid Discord token format: {}", e));
        }

        let intents = GatewayIntents::GUILD_MESSAGES;
        let client = Client::builder(&token, intents)
            .await
            .expect("Error creating Discord client");

        Ok(Self { client })
    }

    pub async fn handle_event(&self, event: SpaceEvent) -> Result<()> {
        let start = Instant::now();
        info!("Handling Discord event: {:?}", event);

        send_discord_message(&self.client, &event).await?;

        let duration = start.elapsed();
        info!("Discord event handled successfully in {:?}", duration);
        Ok(())
    }
}

async fn send_discord_message(client: &Client, event: &SpaceEvent) -> Result<()> {
    let (title, description, color, thumbnail) = match event {
        SpaceEvent::Open => (
            "Noisebridge is Open!",
            "It's time to start hacking.",
            COLOR_OPEN,
            "https://www.noisebridge.net/images/7/7f/Open.png"
        ),
        SpaceEvent::Closed => (
            "Noisebridge is Closed!",
            "We'll see you again soon.",
            COLOR_CLOSED,
            "https://www.noisebridge.net/images/c/c9/Closed.png"
        ),
        SpaceEvent::Initializing => return Ok(()), // Don't send message for initialization
    };

    let channel_id = env::var("DISCORD_CHANNEL_ID")
        .expect("Expected DISCORD_CHANNEL_ID in environment")
        .parse::<u64>()?;

    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
        .thumbnail(thumbnail);

    if let Err(why) = ChannelId::new(channel_id).send_message(&client.http, CreateMessage::default().add_embed(embed)).await {
        error!("Error sending Discord message: {:?}", why);
        return Err(anyhow::anyhow!("Failed to send Discord message: {}", why));
    }

    Ok(())
} 