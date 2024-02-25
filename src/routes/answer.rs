use warp::http::StatusCode;

use crate::store::Store;
use crate::types::account::Session;
use crate::types::answer::{Answer, NewAnswer};

pub async fn add_answer(
    session: Session,
    store: Store,
    new_answer: NewAnswer,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;

    let answer = NewAnswer {
        content: new_answer.content,
        question_id: new_answer.question_id,
    };

    match store.add_answer(answer, account_id).await {
        Ok(_) => Ok(warp::reply::with_status("Answer added", StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

pub async fn update_answer(
    id: i32,
    session: Session,
    store: Store,
    answer: Answer,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_answer_owner(id, &account_id).await? {
        let answer = Answer {
            id: answer.id,
            content: answer.content,
            question_id: answer.question_id,
        };
        match store.update_answer(answer, id, account_id).await {
            Ok(res) => Ok(warp::reply::json(&res)),
            Err(e) => Err(warp::reject::custom(e)),
        }
    } else {
        Err(warp::reject::custom(handle_errors::Error::Unauthorized))
    }
}

pub async fn delete_answer(
    id: i32,
    session: Session,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let account_id = session.account_id;
    if store.is_answer_owner(id, &account_id).await? {
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
