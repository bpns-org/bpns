// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use actix_cors::Cors;
use actix_web::{error, web, App, HttpResponse, HttpServer};
use serde_json::json;

mod handler;

use crate::CONFIG;

#[actix_web::main]
pub async fn run() {
    log::info!("REST API started");

    let _ = HttpServer::new(move || {
        let json_config = web::JsonConfig::default().error_handler(|err, _req| {
            error::InternalError::from_response(
                "",
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": 400,
                    "message": err.to_string(),
                    "data": {}
                })),
            )
            .into()
        });

        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(json_config)
            .configure(init_routes)
    })
    .bind(CONFIG.server.http_addr)
    .unwrap()
    .run()
    .await;
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::ping);
    cfg.service(handler::new_push_notification_token);

    // Subscription
    cfg.service(handler::subscribe);
    cfg.service(handler::unsubscribe);

    // Notifications
    cfg.service(handler::notifications);
    cfg.service(handler::delete_all_notifications);
    cfg.service(handler::delete_notification_by_id);

    // Addresses
    cfg.service(handler::addresses);
    cfg.service(handler::add_addresses);
    cfg.service(handler::delete_addresses);
    cfg.service(handler::add_addresses_from_singlesig);
    cfg.service(handler::delete_addresses_from_singlesig);
    cfg.service(handler::add_addresses_from_multisig);
    cfg.service(handler::delete_addresses_from_multisig);
}
