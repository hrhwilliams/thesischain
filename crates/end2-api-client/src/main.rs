use anyhow::Result;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::Deserialize;

use crate::device::{Device, DeviceInfo, Otk};

mod device;

#[derive(Deserialize)]
struct UserInfo {
    pub id: String,
    pub username: String,
    pub nickname: Option<String>,
}

#[derive(Deserialize)]
struct ChannelInfo {
    pub channel_id: String,
    pub participants: Vec<UserInfo>,
}

#[derive(Deserialize)]
struct ChannelId {
    pub channel_id: String,
}

struct ApiClient {
    client: Client,
    device: Device,
    user_id: String,
    username: String,
    password: String,
}

impl ApiClient {
    async fn new(username: &str, password: &str, device: Option<Device>) -> Result<Self> {
        let client = Client::new();
        let user_info = client
            .post("http://localhost:8081/api/auth/register")
            .json(&serde_json::json!({
                "username": username,
                "password": password,
                "confirm_password": password,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<UserInfo>()
            .await?;

        let device = if device.is_some() {
            // SAFETY: checked in if statement above
            unsafe { device.unwrap_unchecked() }
        } else {
            let device_info = client
                .post("http://localhost:8081/api/me/device")
                .basic_auth(username, Some(password))
                .send()
                .await?
                .error_for_status()?
                .json::<DeviceInfo>()
                .await?;

            let device = Device::new(&device_info.device_id);

            let _ = client
                .post(&format!(
                    "http://localhost:8081/api/me/device/{}",
                    device_info.device_id
                ))
                .basic_auth(username, Some(password))
                .json(&device.get_identity_keys())
                .send()
                .await?
                .error_for_status()?;

            device
        };

        Ok(Self {
            client,
            device,
            user_id: user_info.id,
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    fn get(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .get(format!("http://localhost:8081/api{}", endpoint))
            .basic_auth(&self.username, Some(&self.password))
    }

    fn post(&self, endpoint: &str) -> RequestBuilder {
        self.client
            .post(format!("http://localhost:8081/api{}", endpoint))
            .basic_auth(&self.username, Some(&self.password))
    }

    async fn upload_otks(&mut self) -> Result<()> {
        self.post(&format!("/me/device/{}/otks", self.device.id()))
            .json(&self.device.get_otks(10))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn create_channel(&self, recipient: &str) -> Result<()> {
        self.post("/channel")
            .json(&serde_json::json!({
                "recipient": recipient
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn channels(&self) -> Result<Vec<ChannelId>> {
        let response = self
            .get("/me/channels")
            .send()
            .await?
            .json::<Vec<ChannelId>>()
            .await?;

        Ok(response)
    }

    async fn get_channel_participants(&self, channel: &ChannelId) -> Result<ChannelInfo> {
        let response = self
            .get(&format!("/channel/{}", channel.channel_id))
            .send()
            .await?
            .json::<ChannelInfo>()
            .await?;

        Ok(response)
    }

    async fn get_user_info(&self, user_id: &str) -> Result<UserInfo> {
        let response = self
            .get(&format!("/user/{}", user_id))
            .send()
            .await?
            .json::<UserInfo>()
            .await?;

        Ok(response)
    }

    async fn get_user_devices(&self, user_id: &str) -> Result<Vec<DeviceInfo>> {
        let response = self
            .get(&format!("/user/{}/devices", user_id))
            .send()
            .await?
            .json::<Vec<DeviceInfo>>()
            .await?;

        Ok(response)
    }

    async fn get_device_info(&self, user_id: &str, device_id: &str) -> Result<DeviceInfo> {
        let response = self
            .get(&format!("/user/{}/device/{}", user_id, device_id))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            let text = response.text().await?;
            panic!("{}", text);
        }

        match response.json::<DeviceInfo>().await {
            Ok(info) => return Ok(info),
            Err(e) => panic!("{:?}", e),
        }
    }

    async fn get_device_otk(&self, user_id: &str, device_id: &str) -> Result<Otk> {
        let response = self
            .post(&format!("/user/{}/device/{}/otk", user_id, device_id))
            .send()
            .await?
            .json::<Otk>()
            .await?;

        Ok(response)
    }

    async fn me(&self) -> Result<UserInfo> {
        Ok(self
            .get("/api/me")
            .send()
            .await?
            .error_for_status()?
            .json::<UserInfo>()
            .await?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client1 = ApiClient::new("maki1", "abc", None).await?;
    let mut client2 = ApiClient::new("maki2", "abc", None).await?;

    client1.upload_otks().await?;
    client2.upload_otks().await?;

    client1.create_channel("maki2").await?;

    let channels1 = client1.channels().await?;
    let channel1_id = channels1.first().unwrap();
    let channel1_info = client1.get_channel_participants(channel1_id).await?;
    for participant in channel1_info.participants {
        let user = client1.get_user_info(&participant.id).await?;
        let device_ids = client1.get_user_devices(&participant.id).await?;
        let mut devices = vec![];

        for device in device_ids {
            devices.push(client1.get_device_info(&user.id, &device.device_id).await?);

            if device.device_id != client1.device.id() {
                let otk = client1
                    .get_device_otk(&participant.id, &device.device_id)
                    .await?;
                println!("{}", otk.id);

                // now can actually encrypt
            }
        }
        println!("{:?}", devices);
    }

    client2.channels().await?;

    Ok(())
}
