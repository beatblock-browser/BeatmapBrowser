use actix_web::{post, Responder};
use crate::api::APIError;
use crate::util::data;
use crate::schema::{BeatMap, User};
use crate::util::warp::Replyable;
use crate::schema::delete::DeleteRequest;
use mongodb::bson::doc;
use crate::util::mongo::{MAPS_COLLECTION, USERS_COLLECTION};

pub const ADMINS: [&'static str; 1] = ["gfde6dkqtey5trmfya8h"];

#[post("/api/delete")]
pub async fn delete(
    _req: DeleteRequest,
    mut user: User,
    map: BeatMap,
) -> impl Responder {
    if map.charter_uid != user.id && !ADMINS.contains(&user.id.to_string().as_str()) {
        return Err(APIError::PermissionError().into());
    }

    user.maps.remove(user.maps.iter().position(|elem| elem == &map.id).ok_or(APIError::AlreadyDownloaded())?);
    data().await.database.remove(MAPS_COLLECTION, doc! { "id": map.id.to_string() }).await
        .map_err(APIError::database_error)?;
    let maps_as_strings: Vec<String> = user.maps.iter().map(|id| id.to_string()).collect();
    data().await.database.update(USERS_COLLECTION, doc! { "id": user.id.to_string() }, doc! { "$set": { "maps": maps_as_strings } }).await
        .map_err(APIError::database_error)?;
    // TODO: Remove associated files from storage if needed
    Ok("Ok")
}
