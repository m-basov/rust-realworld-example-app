mod article;
mod comment;
mod error;
mod profile;
mod tag;
mod user;

use server::{warp, ServerState};
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use error::RestError;

pub struct Rest;

impl Rest {
    pub fn new(state: ServerState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        tag::routes(Arc::clone(&state))
            .or(comment::routes(Arc::clone(&state)))
            .or(user::routes(Arc::clone(&state)))
            .or(profile::routes(Arc::clone(&state)))
            .or(article::routes(Arc::clone(&state)))
            .boxed()
    }
}
