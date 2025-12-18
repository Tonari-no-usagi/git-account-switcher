use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;

/// GitHub OAuth App client ID. / GitHub の OAuth App クライアント ID。
/// Used for device flow authentication (browser authentication). / デバイスフローによる認証（ブラウザ認証）に使用されます。
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

/// Initiate the GitHub Device Flow (OAuth 2.0) authentication process.
/// 
/// Obtain the user code for the user to enter in the browser and the device code required for polling.
/// 
/// # Returns
/// Returns the following tuple:
/// - `device_code`: Identifier for polling
/// - `user_code`: 8-digit code the user enters on GitHub
/// - `verification_uri`: URL the user accesses
/// - `interval`: Polling interval (seconds)
///
/// # Errors
/// Returns an error if communication with the GitHub API fails or if the client ID is not configured.
/// -----------------------------------------------------------------------------------------------------
/// GitHub の Device Flow（OAuth 2.0）による認証プロセスを開始します。
/// 
/// ユーザーがブラウザで入力するためのユーザーコードや、ポーリングに必要なデバイスコードを取得します。
/// 
/// # Returns
/// 以下のタプルの結果を返します：
/// - `device_code`: ポーリング用の識別子
/// - `user_code`: ユーザーが GitHub 上で入力する 8 桁のコード
/// - `verification_uri`: ユーザーがアクセスする URL
/// - `interval`: ポーリング間隔（秒）
///
/// # Errors
/// GitHub API との通信に失敗した場合や、クライアント ID が未設定の場合にエラーを返します。
pub fn start_device_flow() -> Result<(String, String, String, u64)> {
    if CLIENT_ID == "YOUR_CLIENT_ID_HERE" {
        bail!("Client ID is not configured in source code.");
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
    let body: DeviceCodeResponse = res.json().context("Failed to parse device code response")?;
    Ok((body.device_code, body.user_code, body.verification_uri, body.interval))
}

/// Attempts to obtain an access token (polling) until the user completes authentication in the browser.
///
/// # Arguments
/// * `device_code` - The device code obtained from `start_device_flow`
/// * `interval` - The recommended polling interval
///
/// # Errors
/// Returns an error if the request times out or the user explicitly denies access.
/// -----------------------------------------------------------------------------------------------------
/// ユーザーがブラウザで認証を完了するまで、アクセストークンの取得を試行（ポーリング）します。
///
/// # Arguments
/// * `device_code` - `start_device_flow` で取得したデバイスコード
/// * `interval` - 推奨されるポーリング間隔
///
/// # Errors
/// タイムアウトした場合や、ユーザーが明示的に拒否した場合にエラーを返します。
pub fn poll_for_token(device_code: &str, interval: u64) -> Result<String> {
    let client = Client::new();
    let url = "https://github.com/login/oauth/access_token";
    let wait_time = Duration::from_secs(interval + 1);

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
                if err == "authorization_pending" { continue; }
                else if err == "slow_down" { thread::sleep(Duration::from_secs(5)); continue; }
                else { bail!("Authorization error: {}", err); }
            }
        } else {
             let status = res.status();
             let error_text = res.text().unwrap_or_default();
             bail!("GitHub Token Error: {} - {}", status, error_text);
        }
    }
    bail!("Timeout waiting for authorization.");
}

/// Retrieves the username from GitHub using the specified access token.
///
/// # Errors
/// Returns an error if the token is invalid or if communication with the API fails.
/// -----------------------------------------------------------------------------------------------------
/// 指定されたアクセストークンを使用して、GitHub からユーザー名を取得します。
///
/// # Errors
/// トークンが無効な場合や、API との通信に失敗した場合にエラーを返します。
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

    let user: UserResponse = res.json().context("Failed to parse user info")?;
    Ok(user.login)
}
