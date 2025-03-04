use warp::http::StatusCode;

use crate::types::account::Session;
use crate::types::answer::{Answer, NewAnswer};
use crate::handle_errors;

pub mod store_trait;
use store_trait::StoreTrait;

#[cfg(test)]
mod tests;

/**
 * @Notice Add answer to a question
 *
 * @Dev This function allows a user to add a new answer to a question in the system.
 *      It requires an authenticated user session and the details of the new answer.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `session`: The authenticated user session object.
 * @params `new_answer`: The details of the new answer to be added.
*/
pub async fn add_answer<S: StoreTrait>(
    session: Session,
    store: S,
    new_answer: NewAnswer,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Extract the account ID from the session for authorization.
    let account_id = session.account_id;
    // Create a new `Answer` object from the provided information.
    // This step might be unnecessary depending on your data structure.
    let answer = NewAnswer {
        content: new_answer.content,
        question_id: new_answer.question_id,
    };
    // Delegate the answer addition to the `store`.
    match store.add_answer(answer, account_id).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Update answer
 *
 * @Dev Allows a user to update an existing answer, provided they are the owner.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `id`: The ID of the answer to be updated.
 * @params `session`: The authenticated user session object.
 * @params `answer`: The updated answer details.
*/
pub async fn update_answer<S: StoreTrait>(
    id: i32,
    session: Session,
    store: S,
    answer: Answer,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Extract the account ID from the session for authorization.
    let account_id = session.account_id;
    // Check if the currently logged-in user owns the answer they're trying to delete.
    if store.is_answer_owner(id, &account_id).await? {
        // Update the answer object with the provided details.
        let answer = Answer {
            id: answer.id,
            content: answer.content,
            question_id: answer.question_id,
        };
        // Delegate the answer update to the `store`.
        match store.update_answer(answer, id, account_id).await {
            Ok(res) => Ok(warp::reply::json(&res)),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

/**
 * @Notice Delete answer
 *
 * @Dev Allows a user to delete an existing answer, provided they are the owner.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `session`: The authenticated user session object.
 * @params `id`: The ID of the answer to be updated.
*/
pub async fn delete_answer<S: StoreTrait>(
    id: i32,
    session: Session,
    store: S,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Extract the account ID from the session for authorization.
    let account_id = session.account_id;
    // Check if the currently logged-in user owns the answer they're trying to delete.
    if store.is_answer_owner(id, &account_id).await? {
        // User is authorized to delete the answer.
        match store.delete_answer(id, account_id).await {
            Ok(_) => Ok(warp::reply::with_status(
                format!("Answer {} deleted", id),
                StatusCode::OK,
            )),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}
