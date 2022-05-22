// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use matrix_sdk::{
    room::{Joined, Room},
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent,
    },
};

use crate::{
    common,
    core::api::{self, CoreApi},
    matrix::MATRIX_STORE,
    util, CONFIG,
};

pub(crate) async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    if *event.sender.clone() == CONFIG.matrix.user_id {
        return;
    }

    if let Room::Joined(room) = room {
        let command = Command::new(event, room.clone());
        if let Err(err) = command.process_command().await {
            log::error!("{:#?}", err);
        };
    }
}

pub(crate) struct Command {
    event: OriginalSyncRoomMessageEvent,
    room: Joined,
}

#[derive(Debug)]
pub(crate) enum Error {
    CoreApi(api::Error),
    Db(common::db::Error),
}

impl Command {
    pub fn new(event: OriginalSyncRoomMessageEvent, room: Joined) -> Self {
        Self { event, room }
    }

    pub async fn process_command(&self) -> Result<(), Error> {
        let msg_body = match &self.event.content.msgtype {
            MessageType::Text(TextMessageEventContent { body, .. }) => body,
            _ => {
                log::warn!("Error getting text message!");
                return Ok(());
            }
        };

        log::debug!("Message received: {}", msg_body);

        let start = Instant::now();

        let user_id: &str = self.event.sender.as_str();

        let msg_splitted: Vec<&str> = msg_body.split(' ').collect();
        let command: &str = msg_splitted[0];

        match command {
            "!addresses" => self.addresses(user_id).await?,
            "!addaddresses" => self.add_addresses(user_id, msg_splitted).await?,
            "!deleteaddresses" => self.delete_addresses(user_id, msg_splitted).await?,
            "!addsinglesig" => self.add_singlesig(user_id, msg_splitted).await?,
            "!deletesinglesig" => self.delete_singlesig(user_id, msg_splitted).await?,
            "!addmultisig" => self.add_multisig(user_id, msg_splitted).await?,
            "!deletemultisig" => self.delete_multisig(user_id, msg_splitted).await?,
            "!subscribe" => self.subscribe(user_id, msg_splitted).await?,
            "!autosubscribe" => self.auto_subscribe(user_id).await?,
            "!unlink" => self.unlink(user_id).await?,
            "!unsubscribe" => self.unsubscribe(user_id).await?,
            "!gettoken" => self.get_token(user_id).await?,
            "!generatetoken" => self.generate_token().await,
            "!help" => self.help().await,
            _ => self.send("Invalid command").await,
        };

        log::trace!(
            "{} command processed in {} ms",
            command,
            start.elapsed().as_millis()
        );

        Ok(())
    }

    async fn addresses(&self, user_id: &str) -> Result<(), Error> {
        if MATRIX_STORE.user_exist(user_id) {
            let user = MATRIX_STORE.get_user(user_id)?;

            let addresses = CoreApi::new(user.token.as_str()).addresses()?;

            let mut msg = format!("Addresses ({})\n", addresses.len());

            addresses.iter().for_each(|address| {
                let address_len: usize = address.len();
                let address_cutted: &str = &address[address_len - 5..address_len];

                let address_formatted: String =
                    format!("- {}...{}\n", &address[0..5], address_cutted);

                msg.push_str(address_formatted.as_str());
            });

            self.send(msg).await;
        } else {
            self.send("This account in not subscribed.").await;
        }

        Ok(())
    }

    async fn add_addresses(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        if msg_splitted.len() >= 2 {
            if MATRIX_STORE.user_exist(user_id) {
                let user = MATRIX_STORE.get_user(user_id)?;
                let addresses: &[&str] = &msg_splitted[1..msg_splitted.len()];

                CoreApi::new(user.token.as_str())
                    .add_addresses(util::convert::vec_to_vec_string(addresses.to_vec()));

                let _ = self.room.redact(&self.event.event_id, None, None).await;

                self.send("Addresses added").await;
            } else {
                self.send("This account in not subscribed.").await;
            }
        } else {
            self.send("Invalid addresses").await;
        }

        Ok(())
    }

    async fn delete_addresses(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        if msg_splitted.len() >= 2 {
            if MATRIX_STORE.user_exist(user_id) {
                let user = MATRIX_STORE.get_user(user_id)?;
                let addresses: &[&str] = &msg_splitted[1..msg_splitted.len()];

                CoreApi::new(user.token.as_str())
                    .delete_addresses(util::convert::vec_to_vec_string(addresses.to_vec()));

                let _ = self.room.redact(&self.event.event_id, None, None).await;
                self.send("Addresses deleted").await;
            } else {
                self.send("This account in not subscribed.").await;
            }
        } else {
            self.send("Invalid addresses").await;
        }

        Ok(())
    }

    async fn add_singlesig(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        if msg_splitted.len() >= 2 {
            if MATRIX_STORE.user_exist(user_id) {
                let user = MATRIX_STORE.get_user(user_id)?;
                let public_key = msg_splitted[1];

                if !public_key.is_empty() {
                    let client = CoreApi::new(user.token.as_str());

                    client.add_addresses_from_singlesig(public_key, 0, 250, false)?;
                    client.add_addresses_from_singlesig(public_key, 0, 250, true)?;

                    let _ = self.room.redact(&self.event.event_id, None, None).await;

                    self.send("Addresses added").await;
                } else {
                    self.send("Invalid public key").await;
                }
            } else {
                self.send("This account in not subscribed.").await;
            }
        } else {
            self.send("Invalid public key").await;
        }

        Ok(())
    }

    async fn delete_singlesig(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        if msg_splitted.len() >= 2 {
            if MATRIX_STORE.user_exist(user_id) {
                let user = MATRIX_STORE.get_user(user_id)?;
                let public_key = msg_splitted[1];

                if !public_key.is_empty() {
                    let client = CoreApi::new(user.token.as_str());

                    client.delete_addresses_from_singlesig(public_key, 0, 250, false)?;
                    client.delete_addresses_from_singlesig(public_key, 0, 250, true)?;

                    let _ = self.room.redact(&self.event.event_id, None, None).await;

                    self.send("Addresses deleted").await;
                } else {
                    self.send("Invalid public key").await;
                }
            } else {
                self.send("This account in not subscribed.").await;
            }
        } else {
            self.send("Invalid public key").await;
        }

        Ok(())
    }

    async fn add_multisig(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        if msg_splitted.len() >= 4 {
            if MATRIX_STORE.user_exist(user_id) {
                let user = MATRIX_STORE.get_user(user_id)?;

                let script: &str = msg_splitted[1];
                let required_signatures: u8 = match msg_splitted[2].parse::<u8>() {
                    Ok(num) => num,
                    Err(_) => {
                        self.send("Invalid n (must be a valid number)").await;
                        return Ok(());
                    }
                };
                let public_keys: &[&str] = &msg_splitted[3..];

                if !script.is_empty() {
                    let client = CoreApi::new(user.token.as_str());

                    client.add_addresses_from_multisig(
                        script,
                        required_signatures,
                        &util::convert::vec_to_vec_string(public_keys.to_vec()),
                        0,
                        250,
                        false,
                    )?;

                    client.add_addresses_from_multisig(
                        script,
                        required_signatures,
                        &util::convert::vec_to_vec_string(public_keys.to_vec()),
                        0,
                        250,
                        true,
                    )?;

                    let _ = self.room.redact(&self.event.event_id, None, None).await;
                    self.send("Addresses added").await;
                } else {
                    self.send("Invalid multisig details").await;
                }
            } else {
                self.send("This account in not subscribed.").await;
            }
        } else {
            self.send("Invalid multisig details").await;
        }

        Ok(())
    }

    async fn delete_multisig(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        if msg_splitted.len() >= 4 {
            if MATRIX_STORE.user_exist(user_id) {
                let user = MATRIX_STORE.get_user(user_id)?;

                let script: &str = msg_splitted[1];
                let public_keys: &[&str] = &msg_splitted[3..];
                let required_signatures: u8 = match msg_splitted[2].parse::<u8>() {
                    Ok(num) => num,
                    Err(_) => {
                        self.send("Invalid n (must be a valid number)").await;
                        return Ok(());
                    }
                };

                if !script.is_empty() {
                    let client = CoreApi::new(user.token.as_str());

                    client.delete_addresses_from_multisig(
                        script,
                        required_signatures,
                        &util::convert::vec_to_vec_string(public_keys.to_vec()),
                        0,
                        250,
                        false,
                    )?;

                    client.delete_addresses_from_multisig(
                        script,
                        required_signatures,
                        &util::convert::vec_to_vec_string(public_keys.to_vec()),
                        0,
                        250,
                        true,
                    )?;

                    let _ = self.room.redact(&self.event.event_id, None, None).await;

                    self.send("Addresses deleted").await;
                } else {
                    self.send("Invalid multisig").await;
                }
            } else {
                self.send("This account in not subscribed.").await;
            }
        } else {
            self.send("Invalid multisig details").await;
        }

        Ok(())
    }

    async fn subscribe(&self, user_id: &str, msg_splitted: Vec<&str>) -> Result<(), Error> {
        let room_id: &str = self.room.room_id().as_str();

        if !MATRIX_STORE.user_with_room_exist(user_id, room_id) {
            if msg_splitted.len() >= 2 {
                let token = msg_splitted[1];

                if !token.is_empty() {
                    CoreApi::new(token).subscribe()?;
                    MATRIX_STORE.create_user(user_id, room_id, token)?;

                    let _ = self.room.redact(&self.event.event_id, None, None).await;

                    self.send("Subscribed").await;
                } else {
                    self.send("Please provide a token.\nTo subscribe send: !subscribe <token>")
                        .await;
                }
            } else {
                self.send("Please provide a token.\nTo subscribe send: !subscribe <token>")
                    .await;
            }
        } else {
            self.send("This account is already subscribed").await;
        }

        Ok(())
    }

    async fn auto_subscribe(&self, user_id: &str) -> Result<(), Error> {
        let room_id: &str = self.room.room_id().as_str();

        if !MATRIX_STORE.user_with_room_exist(user_id, room_id) {
            let token: String = CoreApi::new_push_notification_token();
            let token: &str = token.as_str();

            CoreApi::new(token).subscribe()?;

            MATRIX_STORE.create_user(user_id, room_id, token)?;
            self.send("Subscribed").await;
        } else {
            self.send("This account is already subscribed").await;
        }

        Ok(())
    }

    async fn unlink(&self, user_id: &str) -> Result<(), Error> {
        if MATRIX_STORE.user_exist(user_id) {
            MATRIX_STORE.delete_user(user_id)?;

            self.send("Unlinked").await;
        } else {
            self.send("No token linked to this account").await;
        }

        Ok(())
    }

    async fn unsubscribe(&self, user_id: &str) -> Result<(), Error> {
        if MATRIX_STORE.user_exist(user_id) {
            let user = MATRIX_STORE.get_user(user_id)?;
            CoreApi::new(user.token.as_str()).unsubscribe()?;

            MATRIX_STORE.delete_user(user_id)?;

            self.send("Unsubscribed").await;
        } else {
            self.send("This account is not subscribed").await;
        }

        Ok(())
    }

    async fn get_token(&self, user_id: &str) -> Result<(), Error> {
        if MATRIX_STORE.user_exist(user_id) {
            let user = MATRIX_STORE.get_user(user_id)?;
            self.send(user.token).await;
        } else {
            self.send("This account is not subscribed").await;
        }

        Ok(())
    }

    async fn generate_token(&self) {
        let token: String = CoreApi::new_push_notification_token();

        self.send(token).await;
    }

    async fn help(&self) {
        let mut msg = String::new();
        msg.push_str("!addresses - Get addresses\n");
        msg.push_str("!addaddresses <address> ... - Add addresses\n");
        msg.push_str("!addsinglesig <pubkey> - Add addresses from singlesig\n");
        msg.push_str(
            "!addmultisig <script> <n> <pubkey> <pubkey> ... - Add addresses from multisig\n",
        );
        msg.push_str("!deleteaddresses <address> ... - Delete addresses\n");
        msg.push_str("!deletesinglesig <pubkey> - Delete addresses from singlesig\n");
        msg.push_str(
            "!deletemultisig <script> <n> <pubkey> <pubkey> ... - Delete addresses from multisig\n",
        );
        msg.push_str("!subscribe <token> - Subscribe with token\n");
        msg.push_str("!autosubscribe - Automatic subscribe\n");
        msg.push_str("!unlink - Unlink account from token\n");
        msg.push_str("!unsubscribe - Unsubscribe\n");
        msg.push_str("!gettoken - Get your token\n");
        msg.push_str("!generatetoken - Generate token\n");
        msg.push_str("!help - Help");

        self.send(msg).await;
    }

    async fn send(&self, msg: impl Into<String>) {
        let msg: String = msg.into();

        if !msg.is_empty() {
            let content = RoomMessageEventContent::text_plain(msg);
            if let Err(err) = self.room.send(content, None).await {
                log::error!("Impossible to send message: {:?}", err);
            }
        }
    }
}

impl From<api::Error> for Error {
    fn from(err: api::Error) -> Self {
        Error::CoreApi(err)
    }
}

impl From<common::db::Error> for Error {
    fn from(err: common::db::Error) -> Self {
        Error::Db(err)
    }
}
