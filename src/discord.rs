use std::env;
use std::time::Instant;

use anyhow::Result;
use serenity::all::{prelude::*, Color, CreateEmbed, CreateMessage};
use serenity::model::id::ChannelId;
use tracing::{info, error};

const COLOR_OPEN: Color = Color::new(0x00FF00);    // Green for open
const COLOR_CLOSED: Color = Color::new(0xFF0000);  // Red for closed
const COLOR_STARTUP: Color = Color::new(0xFFA500); // Orange for startup

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
            channel_id: ChannelId::new(channel_id),
        })
    }

    pub async fn send_circuit_event(&self, event: &crate::gpio::CircuitEvent) -> Result<()> {
        let start = Instant::now();
        info!("Sending Discord message for circuit event: {:?}", event);

        let embed = CreateEmbed::new()
            .title(format!("Noisebridge is {}!", event))
            .description(match event {
                crate::gpio::CircuitEvent::Open => "It's time to start hacking.",
                crate::gpio::CircuitEvent::Closed => "We'll see you again soon.",
            })
            .color(match event {
                crate::gpio::CircuitEvent::Open => COLOR_OPEN,
                crate::gpio::CircuitEvent::Closed => COLOR_CLOSED,
            }).thumbnail(match event {
                crate::gpio::CircuitEvent::Open => "https://www.noisebridge.net/images/7/7f/Open.png",
                crate::gpio::CircuitEvent::Closed => "https://www.noisebridge.net/images/c/c9/Closed.png",
            });

        if let Err(why) = self.channel_id.send_message(&self.client.http, CreateMessage::default().add_embed(embed)).await {
            error!("Error sending Discord message: {:?}", why);
            return Err(anyhow::anyhow!("Failed to send Discord message: {}", why));
        }

        let duration = start.elapsed();
        info!("Discord message sent successfully in {:?}", duration);
        Ok(())
    }
    
    pub async fn send_startup_message(&self) -> Result<()> {
        let start = Instant::now();
        info!("Sending Discord startup message");

        let embed = CreateEmbed::new()
            .title("Noisebell is starting up!")
            .description("The Noisebell service is initializing and will begin monitoring the space status.")
            .color(COLOR_STARTUP)
            .thumbnail("https://cats.com/wp-content/uploads/2024/07/Beautiful-red-cat-stretches-and-shows-tongue.jpg");

        if let Err(why) = self.channel_id.send_message(&self.client.http, CreateMessage::default().add_embed(embed)).await {
            error!("Error sending Discord startup message: {:?}", why);
            return Err(anyhow::anyhow!("Failed to send Discord startup message: {}", why));
        }

        let duration = start.elapsed();
        info!("Discord startup message sent successfully in {:?}", duration);
        Ok(())
    }
} 