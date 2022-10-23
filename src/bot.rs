use futures::stream::StreamExt;
use std::{env, error::Error, sync::Arc, sync::Mutex};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Event};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

pub async fn start(mut alb: crate::album::Album) -> anyhow::Result<()> {
    let token = env::var("DISCORD_TOKEN")?;
    let alb = Arc::new(Mutex::new(alb));

    // Use intents to only receive guild message events.

    // A cluster is a manager for multiple shards that by default
    // creates as many shards as Discord recommends.
    let (cluster, mut events) = Cluster::new(
        token.to_owned(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    )
    .await?;
    let cluster = Arc::new(cluster);

    // Start up the cluster.
    let cluster_spawn = Arc::clone(&cluster);

    // Start all shards in the cluster in the background.
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    // HTTP is separate from the gateway, so create a new client.
    let http = Arc::new(HttpClient::new(token));

    // Since we only care about new messages, make the cache only
    // cache new messages.
    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    // Process each event as they come in.
    while let Some((shard_id, event)) = events.next().await {
        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(
            shard_id,
            event,
            Arc::clone(&http),
            Arc::clone(&alb),
        ));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>,
    album: Arc<Mutex<crate::album::Album>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) if msg.content.contains("patate") => {
            http.create_message(msg.channel_id)
                .content("Pong!")?
                .exec()
                .await?;
        }
        Event::MessageCreate(msg) => {
            let link = match album.lock() {
                Ok(mut album) => {
                    if msg.content.len() > 1 {
                        if let Some(link) = album.get_rand_pic(&msg.content[1..]) {
                            Some(link.to_owned())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => None,
            };
            match link {
                Some(link) => {
                    http.create_message(msg.channel_id)
                        .content(&link)?
                        .exec()
                        .await?;
                }
                _ => {}
            }
        }
        Event::ShardConnected(_) => {
            println!("Connected on shard {shard_id}");
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
