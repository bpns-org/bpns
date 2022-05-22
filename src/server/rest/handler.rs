// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::future::{ready, Ready};

use actix_web::{delete, dev::Payload, get, post, web, FromRequest, HttpRequest, HttpResponse};
use serde_json::json;

use crate::{
    common::db::Error::InvalidValue,
    core::{self, api::CoreApi},
};

#[derive(Deserialize)]
pub struct User {
    token: String,
}

impl FromRequest for User {
    type Config = ();
    type Error = HttpResponse;
    type Future = Ready<Result<User, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        if let Some(token) = req.headers().get("BPNS-Auth-Token") {
            if let Ok(token) = token.to_str() {
                if !CoreApi::new(token).is_subscribed() {
                    return ready(Err(HttpResponse::Unauthorized().json(json!({
                        "success": false,
                        "code": 401,
                        "message": "Unauthorized",
                        "data": {}
                    }))));
                }

                return ready(Ok(Self {
                    token: token.into(),
                }));
            }
        }

        ready(Err(HttpResponse::Unauthorized().json(json!({
            "success": false,
            "code": 401,
            "message": "Unauthorized",
            "data": {}
        }))))
    }
}

#[derive(Deserialize)]
pub struct Addresses {
    addresses: Vec<String>,
}

#[derive(Deserialize)]
pub struct SingleSig {
    public_key: String,
    from_index: u32,
    to_index: u32,
    is_change: bool,
}

#[derive(Deserialize)]
pub struct MultiSig {
    script_type: String,
    required_signatures: u8,
    public_keys: Vec<String>,
    from_index: u32,
    to_index: u32,
    is_change: bool,
}

#[get("/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Bitcoin Push Notification Service",
        "data": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
        },
    }))
}

#[get("/newPushNotificationToken")]
pub async fn new_push_notification_token() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Get notification token",
        "data": {
            "token": CoreApi::new_push_notification_token()
        }
    }))
}

#[post("/subscribe/{token}")]
pub async fn subscribe(web::Path(token): web::Path<String>) -> HttpResponse {
    let token: &str = token.as_str();

    match CoreApi::new(token).subscribe() {
        Ok(_) => HttpResponse::Created().json(json!({
            "success": true,
            "code": 200,
            "message": "Subscribed",
            "data": {
                "token": token
            },
        })),
        Err(error) => match error {
            core::api::Error::Db(InvalidValue) => HttpResponse::BadRequest().json(json!({
                "success": false,
                "code": 400,
                "message": "Invalid token",
                "data": {}
            })),
            _ => HttpResponse::InternalServerError().json(json!({
                "success": false,
                "code": 500,
                "message": "Unhandled error",
                "data": {}
            })),
        },
    }
}

#[post("/unsubscribe")]
pub async fn unsubscribe(user: User) -> HttpResponse {
    let token: &str = user.token.as_str();

    match CoreApi::new(token).unsubscribe() {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "code": 200,
            "message": "Unsubscribed",
            "data": {},
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "code": 500,
            "message": "Impossible to unsubscribe",
            "data": {}
        })),
    }
}

#[get("/notifications")]
pub async fn notifications(user: User) -> HttpResponse {
    let token: &str = user.token.as_str();

    match CoreApi::new(token).notifications() {
        Ok(notifications) => HttpResponse::Ok().json(json!({
            "success": true,
            "code": 200,
            "message": "Notifications",
            "data": {
                "notifications": notifications
            },
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "code": 500,
            "message": "Impossible to get notifications",
            "data": {}
        })),
    }
}

#[delete("/notification/{id}")]
pub async fn delete_notification_by_id(
    user: User,
    web::Path(id): web::Path<String>,
) -> HttpResponse {
    let token: &str = user.token.as_str();

    if CoreApi::new(token)
        .delete_notification_by_id(id.as_str())
        .is_err()
    {
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "code": 500,
            "message": "Impossible to delete notification",
            "data": {}
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Notification deleted",
        "data": {}
    }))
}

#[delete("/notifications")]
pub async fn delete_all_notifications(user: User) -> HttpResponse {
    let token: &str = user.token.as_str();

    if CoreApi::new(token).delete_all_notifications().is_err() {
        return HttpResponse::InternalServerError().json(json!({
            "success": false,
            "code": 500,
            "message": "Impossible to delete all notifications",
            "data": {}
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "All notifications deleted",
        "data": {}
    }))
}

#[get("/addresses")]
pub async fn addresses(user: User) -> HttpResponse {
    let token: &str = user.token.as_str();

    // Get addresses from database by token
    match CoreApi::new(token).addresses() {
        Ok(addresses) => HttpResponse::Ok().json(json!({
            "success": true,
            "code": 200,
            "message": "Addresses",
            "data": {
                "addresses": addresses
            },
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "code": 500,
            "message": "Impossible to get addresses",
            "data": {}
        })),
    }
}

#[post("/addresses")]
pub async fn add_addresses(user: User, body: web::Json<Addresses>) -> HttpResponse {
    let token: &str = user.token.as_str();

    let addresses_vector: &Vec<String> = &body.addresses;

    if addresses_vector.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": 400,
            "message": "No addresses provided",
            "data": {}
        }));
    }

    CoreApi::new(token).add_addresses(addresses_vector.to_vec());

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Addresses added",
        "data": {}
    }))
}

#[delete("/addresses")]
pub async fn delete_addresses(user: User, body: web::Json<Addresses>) -> HttpResponse {
    let token: &str = user.token.as_str();

    let addresses_vector: &Vec<String> = &body.addresses;

    if addresses_vector.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": 400,
            "message": "No addresses provided",
            "data": {}
        }));
    }

    CoreApi::new(token).delete_addresses(addresses_vector.to_vec());

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Addresses deleted",
        "data": {}
    }))
}

#[post("/addresses/singlesig")]
pub async fn add_addresses_from_singlesig(user: User, body: web::Json<SingleSig>) -> HttpResponse {
    let token: &str = user.token.as_str();

    if CoreApi::new(token)
        .add_addresses_from_singlesig(
            body.public_key.as_str(),
            body.from_index,
            body.to_index,
            body.is_change,
        )
        .is_err()
    {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": 400,
            "message": "Invalid details",
            "data": {}
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Addresses added",
        "data": {}
    }))
}

#[delete("/addresses/singlesig")]
pub async fn delete_addresses_from_singlesig(
    user: User,
    body: web::Json<SingleSig>,
) -> HttpResponse {
    let token: &str = user.token.as_str();

    if CoreApi::new(token)
        .delete_addresses_from_singlesig(
            body.public_key.as_str(),
            body.from_index,
            body.to_index,
            body.is_change,
        )
        .is_err()
    {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": 400,
            "message": "Invalid details",
            "data": {}
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Addresses deleted",
        "data": {}
    }))
}

#[post("/addresses/multisig")]
pub async fn add_addresses_from_multisig(user: User, body: web::Json<MultiSig>) -> HttpResponse {
    let token: &str = user.token.as_str();

    if CoreApi::new(token)
        .add_addresses_from_multisig(
            body.script_type.as_str(),
            body.required_signatures,
            &body.public_keys,
            body.from_index,
            body.to_index,
            body.is_change,
        )
        .is_err()
    {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": 400,
            "message": "Invalid details",
            "data": {}
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Addresses added",
        "data": {}
    }))
}

#[delete("/addresses/multisig")]
pub async fn delete_addresses_from_multisig(user: User, body: web::Json<MultiSig>) -> HttpResponse {
    let token: &str = user.token.as_str();

    if CoreApi::new(token)
        .delete_addresses_from_multisig(
            body.script_type.as_str(),
            body.required_signatures,
            &body.public_keys,
            body.from_index,
            body.to_index,
            body.is_change,
        )
        .is_err()
    {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "code": 400,
            "message": "Invalid details",
            "data": {}
        }));
    }

    HttpResponse::Ok().json(json!({
        "success": true,
        "code": 200,
        "message": "Addresses deleted",
        "data": {}
    }))
}
