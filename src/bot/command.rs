use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn picture_find_and_send(
    album: Arc<Mutex<crate::album::Album>>,
    msg: Box<MessageCreate>,
    http: Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let link = match album.lock() {
        Ok(mut album) => {
            if let Some(link) = album.get_rand_pic(&msg.content[1..]) {
                Some(link.to_owned())
            } else {
                None
            }
        }
        Err(_) => None,
    };
    Ok(match link {
        Some(link) => {
            http.create_message(msg.channel_id)
                .content(&link)?
                .exec()
                .await?;
        }
        _ => {}
    })
}

pub async fn picture_add(
    msg: Box<MessageCreate>,
    album: &Arc<Mutex<crate::album::Album>>,
    http: &Arc<HttpClient>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut num_added = 0;
    let mut split = msg.content.split(' ');
    let mut response = "Je n'ai rien trouvé à ajouter.".to_owned();
    split.next();
    if let Some(deck_name) = split.next() {
        match album.lock() {
            Ok(mut album) => {
                for att in &msg.attachments {
                    album.add_picture(deck_name, &att.url);
                    num_added += 1;
                }
                match album.save() {
                    Ok(_) => println!("album save sucessful"),
                    Err(_) => eprintln!("failed to save album, dataloss is possible"),
                }
            }
            Err(_) => {}
        }
    }
    if num_added > 0 {
        response = format!("J'ai ajouté {} image·s !", num_added);
    }
    http.create_message(msg.channel_id)
        .content(&response)?
        .exec()
        .await?;
    Ok(())
}
