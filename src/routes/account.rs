use crate::types::account::{AccountUpdate, AccountUpdatePassword, Session};
use crate::store::Store;
use crate::routes::authentication::hash_password;
// update account
pub async fn update_account(
    session: Session,
    store: Store,
    account: AccountUpdate,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    match store.update_account(account_id, account).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

// update password
pub async fn update_password(
    session: Session,
    store: Store,
    password: AccountUpdatePassword
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    let hashed_password = AccountUpdatePassword(hash_password(password.0.as_bytes()));

    match store.update_password(account_id, hashed_password).await {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

// get account information
