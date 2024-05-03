//////////////////////////////////////////////////////
//////////////////////////////////////////////////////
/// foxxisbot by falsefox
/// https://falsefox.dev
/// Created: 2024-05-03
/// Last modified: 2024-05-03
/// 
/// !!!DISCLAIMER!!!
/// I'm just learning rust for the first time. 
/// This bot is incredibly bad and the repository only
/// exists so I can get code review from superior devs.
/////////////////////////////////////////////////////////

use byte_unit::{Byte, Unit};
use dotenvy::dotenv;
use serenity::all::Timestamp;
use serenity::async_trait;
use serenity::builder::{CreateAttachment, CreateEmbed, CreateMessage};
use serenity::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::fs::File;
use std::env;
use tokio::time::sleep;
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.content.starts_with("!ping") {
            if let Err(why) = msg.channel_id.say(&ctx.http, "🏓 Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        if msg.content.starts_with("!sys") {
            let mut sys: System = System::new_all();
            sys.refresh_all();

            let used_mem = Byte::parse_str(sys.used_memory().to_string(), true).unwrap();
            let total_mem = Byte::parse_str(sys.total_memory().to_string(), true).unwrap();

            let memorystr = &format!(
                "{:.3}{}/{:.3}{} used",
                used_mem.get_adjusted_unit(Unit::GB).get_value(),
                used_mem.get_adjusted_unit(Unit::GB).get_unit(),
                total_mem.get_adjusted_unit(Unit::GB).get_value(),
                total_mem.get_adjusted_unit(Unit::GB).get_unit()
            );

            let osstr = &format!(
                "{} {}_64",
                sysinfo::System::long_os_version()
                    .unwrap()
                    .replace("\"", ""),
                sysinfo::System::cpu_arch().unwrap().replace("\"", "")
            );

            let mut cpuvect: Vec<String> = vec![];

            for (i, cpu) in sys.cpus().iter().enumerate() {
                if i == 0 {
                    cpuvect.push(format!("{}", cpu.brand()));
                }

                let cpuusage = cpu.cpu_usage();
                let str = &format!("- {}: {}% usage", cpu.name(), cpuusage.floor());
                cpuvect.push(str.clone());
            }

            let embed = CreateEmbed::new()
                .title("<:computer:1236021552491921439> System Information 📊")
                .description("Foxxis is hosted on a Casio F91 wristwatch in the falkland islands.")
                .fields(vec![
                    (
                        "<:icons8linuxserver50:1236027446474571888> OS",
                        osstr,
                        false,
                    ),
                    (
                        "<:icons8cpu50:1236022790055006298> CPUs",
                        &cpuvect.join("\n"),
                        false,
                    ),
                    (
                        "<:icons8ram55:1236022791464157316> Memory Usage",
                        memorystr,
                        false,
                    ),
                ])
                .timestamp(Timestamp::now());
            let builder = CreateMessage::new().add_embed(embed);

            if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending message: {why:?}");
            }
        }

        if msg.content.starts_with("!info") {
            let embed = CreateEmbed::new()
                .title("Foxxisbot Information")
                .description("Foxxis is an open source bot written in rust.")
                .fields(vec![
                    ("Language", "rust", false),
                    ("Author", "falsefox.dev", false),
                    ("License", "GPL 3.0", false),
                    ("Repository", "https://github.com/false-fox/foxxisbot", false)
                ])
                .timestamp(Timestamp::now());
            let builder = CreateMessage::new().add_embed(embed);

            if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending message: {why:?}");
            }
        }


        if msg.content.starts_with("!help") {
            let embed = CreateEmbed::new()
                .title("Foxxisbot help")
                .fields(vec![
                    ("General commands", "``!help`` - display this menu\n``!info`` - print bot info\n``!sys`` print system info\n``!ping`` - ping the bot", false)
                ])
                .timestamp(Timestamp::now());
            let builder = CreateMessage::new().add_embed(embed);

            if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending message: {why:?}");
            }
        }

        if msg.content.starts_with("hi") {
            let file = File::open("./hi.jpg").await.expect("Error opening file");

            let messagewithimage: CreateAttachment = CreateAttachment::file(&file, "filename.jpg")
                .await
                .expect("Error creating attachment");

            let builder = CreateMessage::new()
                .content("test")
                .add_file(messagewithimage);

            if let Err(why) = msg.channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending message: {why:?}");
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name, shard.id, shard.total
            );
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Here we clone a lock to the Shard Manager, and then move it into a new thread. The thread
    // will unlock the manager and print shards' status on a loop.
    let manager = client.shard_manager.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(30)).await;

            let shard_runners = manager.runners.lock().await;

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id, runner.stage, runner.latency,
                );
            }
        }
    });

    // Start two shards. Note that there is an ~5 second ratelimit period between when one shard
    // can start after another.
    if let Err(why) = client.start_shards(2).await {
        println!("Client error: {why:?}");
    }
}
