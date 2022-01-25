use hyper::{Client, Body, Request};
use hyper::body::{HttpBody, Buf};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use serde_json::json;

use std::collections::HashMap;

use crate::sched::Class;

pub(crate) struct WebhookManager {
    // HashMap<CourseCode, MessageID>
    sent: HashMap<String, String>,
    webhook_url: String,
    client: Client<HttpsConnector<HttpConnector>>,
}

impl WebhookManager {
    pub(crate) fn new(webhook_url: String) -> Self {
        Self {
            sent: HashMap::new(),
            webhook_url,
            client: Client::builder().build::<_, Body>(HttpsConnector::new()),
        }
    }

    pub(crate) async fn send_upcoming(&mut self, class: &Class) {
        let times = format!(
            "`{}-{}`",
            class.start_time().format("%H:%M"),
            class.end_time().format("%H:%M")
        );

        let desc = if let Some(l) = class.link() {
            format!("{} [{}]({})", times, class.course(), l)
        } else {
            format!("{} {} (Link not available)", times, class.course())
        };

        let body = json!({
            "embeds": [
                {
                    "title": "Upcoming Class",
                    "description": desc,
                }
            ]
        });

        let uri = format!("{}?wait=true", self.webhook_url);

        let request = Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = self.client.request(request).await.unwrap();

        const MAX_RESPONSE_SIZE: u64 = 8 * 1024; // 8MB

        if response.body().size_hint().upper().unwrap_or(MAX_RESPONSE_SIZE) > MAX_RESPONSE_SIZE {
            // Too big, move on
            return;
        }

        let resp_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        //let resp_json: serde_json::Value = serde_json::from_slice(&resp_bytes[..]).unwrap();
        let mut resp_json: serde_json::Value = serde_json::from_reader(resp_bytes.reader()).unwrap();

        if let serde_json::Value::String(msg_id) = resp_json["id"].take() {
            self.sent.insert(class.course().to_string(), msg_id);
        }

    }

    pub(crate) async fn set_starting(&mut self, class: &Class) -> Option<()> {

        let msg_id = self.sent.get(class.course())?;

        let times = format!(
            "`{}-{}`",
            class.start_time().format("%H:%M"),
            class.end_time().format("%H:%M")
        );

        let desc = if let Some(l) = class.link() {
            format!("{} [{}]({})", times, class.course(), l)
        } else {
            format!("{} {} (Link not available)", times, class.course())
        };

        let body = json!({
            "embeds": [
                {
                    "title": "Current Class",
                    "description": desc,
                }
            ]
        });

        let uri = format!("{}/messages/{}", self.webhook_url, msg_id);

        let request = Request::patch(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();

        let response = self.client.request(request).await.unwrap();
        Some(())
    }

    pub(crate) async fn delete(&mut self, class: &Class) -> Option<()> {
        let msg_id = self.sent.remove(class.course())?;

        let url = format!("{}/messages/{}", self.webhook_url.trim_end_matches('/'), msg_id);

        let request = Request::delete(url).body(Body::from("")).unwrap();

        let response = self.client.request(request).await.ok()?;

        if response.status().as_u16() != 204 {
            return None;
        }

        Some(())
    }

    pub(crate) async fn delete_all(&mut self) -> Option<()> {
        for (_, msg_id) in self.sent.drain() {
            let url = format!("{}/messages/{}", self.webhook_url.trim_end_matches('/'), msg_id);

            let request = Request::delete(url).body(Body::from("")).unwrap();

            let response = self.client.request(request).await.ok()?;

            if response.status().as_u16() != 204 {
                return None;
            }
        }
        Some(())
    }
}
