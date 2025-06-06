use actix_web::{post, Responder};
use crate::api::APIError;
use crate::util::data;
use crate::util::warp::Replyable;
use crate::schema::downloaded::DownloadedRequest;
use mongodb::bson::doc;
use crate::schema::{BeatMap, User};
use crate::util::mongo::USERS_COLLECTION;

#[post("/api/download")]
pub async fn download(
    _req: DownloadedRequest,
    user: User,
    map: BeatMap
) -> impl Responder {
    if !user.downloaded.contains(&map.id) {
        let mut new_downloaded = user.downloaded.clone();
        new_downloaded.push(map.id);
        let new_downloaded_str: Vec<String> = new_downloaded.iter().map(|id| id.to_string()).collect();
        data().await.database.update(USERS_COLLECTION, doc! { "id": user.id.to_string() }, doc! { "$set": { "downloaded": new_downloaded_str } }).await.map_err(APIError::database_error)?;
    }
    Ok("Ok")
}

#[post("/api/remove")]
pub async fn remove(
    _req: DownloadedRequest,
    mut user: User,
    map: BeatMap
) -> impl Responder {
    if let Some(pos) = user.downloaded.iter().position(|elem| elem == &map.id) {
        user.downloaded.remove(pos);
        let downloaded_str: Vec<String> = user.downloaded.iter().map(|id| id.to_string()).collect();
        data().await.database.update(USERS_COLLECTION, doc! { "id": user.id.to_string() }, doc! { "$set": { "downloaded": downloaded_str } }).await.map_err(APIError::database_error)?;
    }
    Ok("Ok")
}
