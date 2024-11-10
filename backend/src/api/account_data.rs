use crate::api::{APIError, AuthenticatedRequest};
use crate::util::ratelimiter::{SiteAction, UniqueIdentifier};
use crate::util::{collect_stream, get_user, LockResultExt};
use crate::SiteData;
use firebase_auth::FirebaseUser;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::Request;
use std::ops::Deref;

pub async fn account_data(
    request: Request<Incoming>,
    identifier: UniqueIdentifier,
    data: &SiteData,
) -> Result<String, APIError> {
    if data
        .ratelimiter
        .lock()
        .ignore_poison()
        .check_limited(SiteAction::UpvoteList, &identifier)
    {
        return Err(APIError::Ratelimited());
    }

    let request_data = collect_stream(request.into_data_stream(), 5000)
        .await
        .map_err(|err| APIError::QueryError(err))?;
    let string = String::from_utf8_lossy(request_data.deref());
    let arguments = serde_json::from_str::<AuthenticatedRequest>(string.deref())
        .map_err(|err| APIError::QueryError(err.into()))?;

    let user: FirebaseUser = data
        .auth
        .verify(&arguments.firebase_token)
        .map_err(|err| APIError::AuthError(err.to_string()))?;
    let user = get_user(true, user.user_id, &data.db).await?;
    Ok(serde_json::to_string(&user)?)
}
