use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;

// ★設定済みのClient IDをそのまま使用してください
pub const CLIENT_ID: &str = "Ov23li6WaAMnOZW2RXsa"; 

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    interval: u64,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct UserResponse {
    login: String,
}

pub fn start_device_flow() -> Result<(String, String, String, u64)> {
    if CLIENT_ID == "YOUR_CLIENT_ID_HERE" {
        bail!("Client ID is not configured in source code. Please edit src/github_auth.rs");
    }

    let client = Client::new();
    let res = client.post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", "repo read:user")])
        .send()
        .context("Failed to connect to GitHub")?;

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().unwrap_or_default();
        bail!("GitHub API Error: {} - {}", status, error_text);
    }

    let body: DeviceCodeResponse = res.json()
        .context("Failed to parse device code response")?;

    Ok((body.device_code, body.user_code, body.verification_uri, body.interval))
}

pub fn poll_for_token(device_code: &str, interval: u64) -> Result<String> {
    let client = Client::new();
    let url = "https://github.com/login/oauth/access_token";
    
    // GitHubの推奨インターバル + 少し余裕を持たせる
    let wait_time = Duration::from_secs(interval + 1);

    // 修正: 回数を20回(約2分) -> 100回(約10分)に延長
    // これでブラウザ操作に時間がかかっても大丈夫になります
    for _ in 0..100 {
        thread::sleep(wait_time);

        let res = client.post(url)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
            ])
            .send()?;

        if res.status().is_success() {
            let body: AccessTokenResponse = res.json()?;
            
            if let Some(token) = body.access_token {
                return Ok(token);
            }
            
            if let Some(err) = body.error {
                if err == "authorization_pending" {
                    continue; // まだ承認されていない（待機継続）
                } else if err == "slow_down" {
                    // GitHubから「頻度が高すぎる」と言われたら長めに待つ
                    thread::sleep(Duration::from_secs(5));
                    continue;
                } else if err == "expired_token" {
                    bail!("Device code expired.");
                } else {
                    bail!("Authorization error: {}", err);
                }
            }
        } else {
             let status = res.status();
             let error_text = res.text().unwrap_or_default();
             bail!("GitHub Token Error: {} - {}", status, error_text);
        }
    }
    bail!("Timeout waiting for authorization.");
}

pub fn get_username(token: &str) -> Result<String> {
    let client = Client::new();
    let res = client.get("https://api.github.com/user")
        .header("User-Agent", "gas-cli")
        .header("Authorization", format!("token {}", token))
        .send()
        .context("Failed to get user info")?;

    if !res.status().is_success() {
        bail!("User Info Error: {}", res.status());
    }

    let user: UserResponse = res.json()
        .context("Failed to parse user info")?;
        
    Ok(user.login)
}