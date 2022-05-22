// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

pub struct Notification {
    address: String,
    txid: String,
    amount: String,
    confirmed: bool,
}

impl Notification {
    pub fn new<T>(address: T, txid: T, amount: T, confirmed: bool) -> Self
    where
        T: Into<String> + std::fmt::Display,
    {
        Self {
            address: address.into(),
            txid: txid.into(),
            amount: amount.into(),
            confirmed,
        }
    }

    // pub fn send(&self) {}

    pub fn as_plain_text(&self) -> String {
        let mut msg: String = String::new();

        msg.push_str(
            format!(
                "New {} transaction {}{}",
                if self.confirmed {
                    "confirmed"
                } else {
                    "pending"
                },
                if self.confirmed { "✅" } else { "⏳" },
                "\n"
            )
            .as_str(),
        );
        msg.push_str(format!("- address: {}{}", self.address, "\n").as_str());
        msg.push_str(format!("- txid: {}{}", self.txid, "\n").as_str());
        msg.push_str(format!("- amount: {}", self.amount).as_str());

        msg
    }

    pub fn as_html(&self) -> String {
        let mut msg: String = String::new();

        msg.push_str(
            format!(
                "New <b>{}</b> transaction {}{}",
                if self.confirmed {
                    "confirmed"
                } else {
                    "pending"
                },
                if self.confirmed { "✅" } else { "⏳" },
                "<br>"
            )
            .as_str(),
        );
        msg.push_str(format!("- address: {}{}", self.address, "<br>").as_str());
        msg.push_str(format!("- txid: {}{}", self.txid, "<br>").as_str());
        msg.push_str(format!("- amount: {}", self.amount).as_str());

        msg
    }
}
