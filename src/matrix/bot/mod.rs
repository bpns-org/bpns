// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use matrix_sdk::{
    config::SyncSettings,
    ruma::{events::room::message::RoomMessageEventContent, RoomId, UserId},
    store::{CryptoStore, StateStore},
    Client, ClientBuilder, Session,
};
use tokio::time::{sleep, Duration};
use tokio_stream::StreamExt;

mod autojoin;
mod commands;
mod notification;

use crate::{
    common,
    core::api::CoreApi,
    matrix::{bot::notification::Notification, util, MATRIX_STORE},
    CONFIG,
};

pub struct Bot;

#[derive(Debug)]
pub enum Error {
    Db(common::db::Error),
    Matrix(matrix_sdk::Error),
    MatrixClientBuilder(matrix_sdk::ClientBuildError),
    MatrixStore(matrix_sdk::StoreError),
    MatrixCryptoStore(matrix_sdk::store::OpenStoreError),
}

impl Bot {
    pub async fn run() -> Result<(), Error> {
        let homeserver_url: &str = CONFIG.matrix.homeserver_url.as_str();
        let user_id: &str = CONFIG.matrix.user_id.as_str();
        let password: &str = CONFIG.matrix.password.as_str();

        let user_id_boxed = Box::<UserId>::try_from(user_id).unwrap();
        let state_store = StateStore::open_with_path(&CONFIG.matrix.state_path)?;
        let crypto_store = CryptoStore::open_with_passphrase(&CONFIG.matrix.state_path, None)?;

        let mut client_builder: ClientBuilder = Client::builder()
            .homeserver_url(homeserver_url)
            .crypto_store(Box::new(crypto_store))
            .state_store(Box::new(state_store));

        if let Some(proxy) = &CONFIG.matrix.proxy {
            client_builder = client_builder.proxy(proxy);
        }

        let client: Client = client_builder.build().await?;

        log::debug!("Checking session...");

        if MATRIX_STORE.session_exist(user_id) {
            let session_store = MATRIX_STORE.get_session(user_id)?;

            let session = Session {
                access_token: session_store.access_token,
                user_id: user_id_boxed,
                device_id: session_store.device_id.into(),
            };

            client.restore_login(session).await?;

            log::debug!("Session restored from database");
        } else {
            log::debug!("Session not found into database");
            log::debug!("Login with credentials...");
            let username = user_id_boxed.localpart();
            client.login(username, password, None, Some("BPNS")).await?;

            log::debug!("Getting session data...");

            if let Some(session) = client.session().await {
                log::debug!("Saving session data into database...");
                MATRIX_STORE.create_session(
                    user_id,
                    &session.access_token,
                    &session.device_id.to_string(),
                )?;

                log::debug!("Session saved to database");
            } else {
                log::error!("Impossible to get and save session");
                log::warn!("The bot can continue to work without saving the session but if you are using an encrypted room, on the next restart, the bot will not be able to read the messages");
            }
        }

        client.account().set_display_name(Some("BPNS")).await?;

        log::info!("Matrix Bot started");

        client
            .register_event_handler(autojoin::on_stripped_state_member)
            .await
            .register_event_handler(commands::on_room_message)
            .await;

        Self::process_pending_notifications(client.clone());

        let settings = SyncSettings::default().full_state(true);
        client.sync(settings).await;

        Ok(())
    }

    fn process_pending_notifications(bot: Client) {
        tokio::spawn(async move {
            loop {
                let mut users = match MATRIX_STORE.get_users() {
                    Ok(result) => tokio_stream::iter(result),
                    Err(error) => {
                        log::error!("Impossible to get users from db: {:?}", error);
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

                while let Some(user) = users.next().await {
                    let room_id = user.room_id;
                    let token = user.token.as_str();

                    let api_client: CoreApi = CoreApi::new(token);
                    let notifications = match api_client.notifications() {
                        Ok(result) => result,
                        Err(error) => {
                            log::error!("Notification processor: {:?}", error);
                            sleep(Duration::from_secs(60)).await;
                            continue;
                        }
                    };
                    let mut stream = tokio_stream::iter(notifications);

                    while let Some(notification) = stream.next().await {
                        log::debug!("{:#?}", notification);

                        let address_cutted: &str =
                            &notification.address[notification.address.len() - 5..];
                        let txid_cutted: &str = &notification.txid[notification.txid.len() - 5..];
                        let amount: String = format!(
                            "{}{}",
                            if notification.txtype == "in" {
                                "+"
                            } else {
                                "-"
                            },
                            util::format_sats(notification.amount)
                        );

                        let address: String =
                            format!("{}...{}", &notification.address[..5], address_cutted);
                        let txid: String = format!("{}...{}", &notification.txid[..5], txid_cutted);

                        let new_notification =
                            Notification::new(address, txid, amount, notification.confirmed);

                        let room_id = Box::<RoomId>::try_from(room_id.as_str()).unwrap();
                        let room = bot.get_joined_room(&room_id).unwrap();
                        let content = RoomMessageEventContent::text_html(
                            new_notification.as_plain_text(),
                            new_notification.as_html(),
                        );

                        match room.send(content, None).await {
                            Ok(_) => {
                                log::info!(
                                    "Sent notification for txid {} ({} - {})",
                                    notification.txid,
                                    notification.txtype,
                                    if notification.confirmed {
                                        "confirmed"
                                    } else {
                                        "pending"
                                    }
                                );

                                match api_client.delete_notification_by_id(notification.id.as_str())
                                {
                                    Ok(_) => {
                                        log::trace!("Notification {} deleted", notification.id)
                                    }
                                    Err(error) => log::error!(
                                        "Impossible to delete notification {}: {:?}",
                                        notification.id,
                                        error
                                    ),
                                }
                            }
                            Err(_) => log::error!("Impossible to send notification"),
                        };

                        sleep(Duration::from_millis(100)).await;
                    }
                }

                sleep(Duration::from_secs(60)).await;
            }
        });
    }
}

impl From<common::db::Error> for Error {
    fn from(err: common::db::Error) -> Self {
        Error::Db(err)
    }
}

impl From<matrix_sdk::Error> for Error {
    fn from(err: matrix_sdk::Error) -> Self {
        Error::Matrix(err)
    }
}

impl From<matrix_sdk::ClientBuildError> for Error {
    fn from(err: matrix_sdk::ClientBuildError) -> Self {
        Error::MatrixClientBuilder(err)
    }
}

impl From<matrix_sdk::StoreError> for Error {
    fn from(err: matrix_sdk::StoreError) -> Self {
        Error::MatrixStore(err)
    }
}

impl From<matrix_sdk::store::OpenStoreError> for Error {
    fn from(err: matrix_sdk::store::OpenStoreError) -> Self {
        Error::MatrixCryptoStore(err)
    }
}
