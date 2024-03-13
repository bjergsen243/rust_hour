use std::collections::HashMap;

use tracing::{event, instrument, Level};
use warp::http::StatusCode;

use crate::store::Store;
use crate::types::account::Session;
use crate::types::pagination::{extract_pagination, Pagination};
use crate::types::question::{NewQuestion, Question};

/**
 * @Notice Get questions
 *
 * @Dev Retrieves questions, with optional pagination.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `params`: Query parameters for pagination.
*/
#[instrument]
pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "rust_hour", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();

    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }

    match store
        .get_questions(pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Update question
 *
 * @Dev Allows a user to update an existing question, provided they are the owner.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `id`: The ID of the question to be updated.
 * @params `session`: The authenticated user session object.
 * @params `question`: The updated question details.
*/
pub async fn update_question(
    id: i32,
    session: Session,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        let question = Question {
            id: question.id,
            title: question.title,
            content: question.content,
            tags: question.tags,
        };
        match store.update_question(question, id, account_id).await {
            Ok(res) => Ok(warp::reply::json(&res)),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

/**
 * @Notice Delete question
 *
 * @Dev Allows a user to delete an existing question, provided they are the owner.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `session`: The authenticated user session object.
 * @params `id`: The ID of the question to be deleted.
*/
pub async fn delete_question(
    id: i32,
    session: Session,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_question_owner(id, &account_id).await? {
        match store.delete_question(id, account_id).await {
            Ok(_) => Ok(warp::reply::with_status(
                format!("Question {} deleted", id),
                StatusCode::OK,
            )),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

/**
 * @Notice Add question
 *
 * @Dev This function allows a user to add a new question in the system.
 *      It requires an authenticated user session and the details of the new question.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `session`: The authenticated user session object.
 * @params `new_question`: The details of the new question to be added.
*/
pub async fn add_question(
    session: Session,
    store: Store,
    new_question: NewQuestion,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;

    let question = NewQuestion {
        title: new_question.title,
        content: new_question.content,
        tags: new_question.tags,
    };

    match store.add_question(question, account_id).await {
        Ok(question) => Ok(warp::reply::json(&question)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/**
 * @Notice Get answers of question
 *
 * @Dev Retrieves answers for a specific question, with optional pagination.
 *
 * @params  `store`: A `Store` instance used to interact with the database.
 * @params `id`: The ID of the question
 * @params `params`: Query parameters for pagination.
*/
#[instrument]
pub async fn get_answers(
    id: i32,
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    event!(target: "rust_hour", Level::INFO, "querying questions");
    let mut pagination = Pagination::default();

    if !params.is_empty() {
        event!(Level::INFO, pagination = true);
        pagination = extract_pagination(params)?;
    }

    match store
        .get_answers(id, pagination.limit, pagination.offset)
        .await
    {
        Ok(res) => Ok(warp::reply::json(&res)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}
