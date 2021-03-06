use crate::RestError;
use conduit::{
    ConduitError, LoginParams, RegisterParams, UpdateUserParams, User, UserDto, UserService,
};
use serde::{Deserialize, Serialize};
use server::{auth, warp, with_state, ServerError, ServerState};
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};

pub fn routes(state: ServerState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    // GET /user
    let get_user = warp::path!("user")
        .and(warp::get())
        .and(with_state(Arc::clone(&state)))
        .and(auth(Arc::clone(&state)))
        .and_then(get_user_handler)
        .boxed();

    // PUT /user
    let update_user = warp::path!("user")
        .and(warp::put())
        .and(warp::body::json())
        .and(with_state(Arc::clone(&state)))
        .and(auth(Arc::clone(&state)))
        .and_then(update_user_handler)
        .boxed();

    // POST /users/login
    let login = warp::path!("users" / "login")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(Arc::clone(&state)))
        .and_then(login_handler)
        .boxed();

    // POST /users
    let register = warp::path!("users")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(Arc::clone(&state)))
        .and_then(register_handler)
        .boxed();

    get_user.or(update_user).or(login).or(register).boxed()
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub user: UserDto,
}

impl From<UserDto> for UserResponse {
    fn from(user: UserDto) -> Self {
        UserResponse { user }
    }
}

async fn get_user_handler(state: ServerState, user: User) -> Result<impl Reply, Rejection> {
    let state = state.read().await;
    let token = state
        .jwt
        .sign(&user)
        .map_err(|err| RestError::from(ServerError::from(err)))?;
    let user_dto = UserDto::with_token(user, token);
    Ok(warp::reply::json(&UserResponse::from(user_dto)))
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    user: LoginParams,
}

async fn login_handler(payload: LoginPayload, state: ServerState) -> Result<impl Reply, Rejection> {
    let state = state.read().await;

    let user = UserService::new(&state.db_pool)
        .login(&payload.user, |user| {
            state
                .jwt
                .sign(user)
                .map_err(|_| ConduitError::CreateTokenError)
        })
        .await
        .map_err(RestError::from)?;

    let set_cookie = auth::set_cookie_token(&user.token);
    Ok(warp::reply::with_header(
        warp::reply::json(&UserResponse::from(user)),
        warp::http::header::SET_COOKIE,
        set_cookie,
    ))
}

#[derive(Debug, Deserialize)]
struct RegisterPayload {
    user: RegisterParams,
}

async fn register_handler(
    payload: RegisterPayload,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let state = state.read().await;

    let user = UserService::new(&state.db_pool)
        .register(&payload.user, |user| {
            state
                .jwt
                .sign(user)
                .map_err(|_| ConduitError::CreateTokenError)
        })
        .await
        .map_err(RestError::from)?;

    let set_cookie = auth::set_cookie_token(&user.token);
    let json = warp::reply::json(&UserResponse::from(user));
    Ok(warp::reply::with_header(
        warp::reply::with_status(json, warp::http::StatusCode::CREATED),
        warp::http::header::SET_COOKIE,
        set_cookie,
    ))
}

#[derive(Deserialize, Debug)]
struct UpdateUserPayload {
    user: UpdateUserParams,
}

async fn update_user_handler(
    payload: UpdateUserPayload,
    state: ServerState,
    user: User,
) -> Result<impl Reply, Rejection> {
    let state = state.read().await;

    let updated_user = UserService::new(&state.db_pool)
        .update_user(&payload.user, &user, |user| {
            state
                .jwt
                .sign(user)
                .map_err(|_| ConduitError::CreateTokenError)
        })
        .await
        .map_err(RestError::from)?;

    Ok(warp::reply::json(&UserResponse::from(updated_user)))
}
