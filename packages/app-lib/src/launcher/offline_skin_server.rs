//! Local skin server for offline accounts to enable in-game skins via authlib-injector
use std::path::PathBuf;
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::{EncodePublicKey, LineEnding};
use rsa::signature::{SignatureEncoding, Signer};
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sqlx::Row;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::state::DirectoryInfo;
use crate::State;

const AUTHLIB_INJECTOR_URL_PRIMARY: &str =
	"https://authlib-injector.yushi.moe/artifact/56/authlib-injector-1.2.8.jar";
const AUTHLIB_INJECTOR_URL_MIRROR: &str =
	"https://bmclapi2.bangbang93.com/mirrors/authlib-injector/artifact/56/authlib-injector-1.2.8.jar";

static SKIN_SERVER_INSTANCE: OnceCell<Arc<SkinServer>> = OnceCell::const_new();

pub struct SkinServer {
	port: u16,
	public_key_pem: String,
	signing_key: SigningKey<Sha1>,
}

#[derive(Serialize)]
struct AuthMetadata<'a> {
	meta: AuthMetadataMeta<'a>,
	#[serde(rename = "skinDomains")]
	skin_domains: Vec<&'a str>,
	#[serde(rename = "signaturePublickey")]
	signature_public_key_lower: &'a str,
	#[serde(rename = "signaturePublicKey")]
	signature_public_key_camel: &'a str,
}

#[derive(Serialize)]
struct AuthMetadataMeta<'a> {
	#[serde(rename = "serverName")]
	server_name: &'a str,
	#[serde(rename = "implementationName")]
	implementation_name: &'a str,
	version: &'a str,
}

#[derive(Serialize, Deserialize)]
struct ProfileResponse {
	id: String,
	name: String,
	properties: Vec<ProfileProperty>,
}

#[derive(Serialize, Deserialize)]
struct ProfileProperty {
	name: String,
	value: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	signature: Option<String>,
}

#[derive(Serialize)]
struct TexturesPayload<'a> {
	timestamp: i64,
	#[serde(rename = "profileId")]
	profile_id: &'a str,
	#[serde(rename = "profileName")]
	profile_name: &'a str,
	textures: TexturesMap<'a>,
}

#[derive(Serialize)]
struct TexturesMap<'a> {
	#[serde(rename = "SKIN")]
	skin: SkinTextureEntry<'a>,
}

#[derive(Serialize)]
struct SkinTextureEntry<'a> {
	url: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	metadata: Option<SkinMetadata<'a>>,
}

#[derive(Serialize)]
struct SkinMetadata<'a> {
	model: &'a str,
}

impl SkinServer {
	pub async fn get_or_init() -> crate::Result<Arc<Self>> {
		SKIN_SERVER_INSTANCE
			.get_or_try_init(|| async {
				ensure_tables().await;

				let (private_key, public_key_pem) = tokio::task::spawn_blocking(|| {
					let mut rng = rand::rngs::OsRng;
					let private_key = RsaPrivateKey::new(&mut rng, 2048)
						.map_err(|e| crate::ErrorKind::OtherError(format!("Failed to generate RSA key: {e}")))?;
					let public_key = RsaPublicKey::from(&private_key);
					let public_key_pem = public_key
						.to_public_key_pem(LineEnding::LF)
						.map_err(|e| crate::ErrorKind::OtherError(format!("Failed to encode RSA public key: {e}")))?;
					Ok::<_, crate::Error>((private_key, public_key_pem))
				})
				.await
				.map_err(|e| crate::ErrorKind::OtherError(format!("RSA keygen task failed: {e}")))?
				?;

				let signing_key = SigningKey::<Sha1>::new(private_key);

				let listener = TcpListener::bind("127.0.0.1:0")
					.await
					.map_err(|e| crate::ErrorKind::OtherError(format!("Failed to bind local skin server: {e}")))?;
				let port = listener
					.local_addr()
					.map_err(|e| crate::ErrorKind::OtherError(format!("Failed to get local skin server port: {e}")))?
					.port();

				let server = Arc::new(Self {
					port,
					public_key_pem,
					signing_key,
				});

				let server_clone = Arc::clone(&server);
				tokio::spawn(async move {
					server_clone.run_listener(listener).await;
				});

				tracing::info!(port = port, "Offline Minecraft skin server started");
				Ok(server)
			})
			.await
			.cloned()
	}

	pub fn port(&self) -> u16 {
		self.port
	}

	async fn run_listener(self: Arc<Self>, listener: TcpListener) {
		loop {
			match listener.accept().await {
				Ok((stream, _)) => {
					let server = Arc::clone(&self);
					tokio::spawn(async move {
						if let Err(e) = server.handle_connection(stream).await {
							tracing::debug!("Error handling skin server request: {e}");
						}
					});
				}
				Err(e) => {
					tracing::error!("Offline skin server accept error: {e}");
					break;
				}
			}
		}
	}

	async fn handle_connection(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let mut buffer = [0u8; 4096];
		let n = stream.read(&mut buffer).await?;
		if n == 0 {
			return Ok(());
		}

		let req_text = String::from_utf8_lossy(&buffer[..n]);
		let mut lines = req_text.lines();
		let request_line = match lines.next() {
			Some(line) => line,
			None => return Ok(()),
		};

		let parts: Vec<&str> = request_line.split_whitespace().collect();
		if parts.len() < 2 {
			return Ok(());
		}

		let method = parts[0];
		let raw_path = parts[1];
		let path = raw_path.split('?').next().unwrap_or(raw_path);

		if method == "POST" && (path.ends_with("/join") || path == "/sessionserver/session/minecraft/join" || path == "/session/minecraft/join") {
			let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
			stream.write_all(resp.as_bytes()).await?;
			return Ok(());
		}

		if method != "GET" {
			let resp = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n";
			stream.write_all(resp.as_bytes()).await?;
			return Ok(());
		}

		if path == "/" || path == "/yggdrasil" {
			self.handle_metadata(&mut stream).await?;
		} else if let Some(uuid_str) = path.strip_prefix("/sessionserver/session/minecraft/profile/").or_else(|| path.strip_prefix("/session/minecraft/profile/")) {
			self.handle_profile(&mut stream, uuid_str).await?;
		} else if path.ends_with("/hasJoined") || path.contains("/hasJoined") {
			self.handle_has_joined(&mut stream, raw_path).await?;
		} else if let Some(texture_key) = path.strip_prefix("/texture/") {
			self.handle_texture(&mut stream, texture_key).await?;
		} else {
			let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
			stream.write_all(resp.as_bytes()).await?;
		}

		Ok(())
	}

	async fn handle_metadata(&self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let metadata = AuthMetadata {
			meta: AuthMetadataMeta {
				server_name: "LeetMC",
				implementation_name: "leetmc-skin-server",
				version: "1.0.0",
			},
			skin_domains: vec![
				"127.0.0.1",
				"localhost",
				".minecraft.net",
				".mojang.com",
				".ely.by",
				".modrinth.com",
				".namemc.com",
			],
			signature_public_key_lower: &self.public_key_pem,
			signature_public_key_camel: &self.public_key_pem,
		};

		let body = serde_json::to_vec(&metadata)?;
		let resp = format!(
			"HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nX-Authlib-Injector-API-Location: http://127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
			self.port,
			body.len()
		);

		stream.write_all(resp.as_bytes()).await?;
		stream.write_all(&body).await?;
		Ok(())
	}

	async fn send_profile_response(
		&self,
		stream: &mut TcpStream,
		clean_uuid_str: &str,
		username: &str,
		user_id: Uuid,
		state: &State,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let (texture_key, is_slim) = get_equipped_skin_for_offline_user(user_id, username, state).await;

		let skin_url = format!("http://127.0.0.1:{}/texture/{}", self.port, texture_key);
		let metadata = if is_slim {
			Some(SkinMetadata { model: "slim" })
		} else {
			None
		};

		let textures_payload = TexturesPayload {
			timestamp: chrono::Utc::now().timestamp_millis(),
			profile_id: clean_uuid_str,
			profile_name: username,
			textures: TexturesMap {
				skin: SkinTextureEntry {
					url: skin_url,
					metadata,
				},
			},
		};

		let payload_json = serde_json::to_string(&textures_payload)?;
		let value = BASE64.encode(payload_json.as_bytes());

		let signature_bytes = self.signing_key.sign(value.as_bytes());
		let signature = BASE64.encode(signature_bytes.to_vec());

		let profile = ProfileResponse {
			id: clean_uuid_str.to_string(),
			name: username.to_string(),
			properties: vec![ProfileProperty {
				name: "textures".to_string(),
				value,
				signature: Some(signature),
			}],
		};

		let body = serde_json::to_vec(&profile)?;
		let resp = format!(
			"HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
			body.len()
		);

		stream.write_all(resp.as_bytes()).await?;
		stream.write_all(&body).await?;
		tracing::info!(uuid = clean_uuid_str, username = username, texture_key = %texture_key, "Served offline skin profile to game");
		Ok(())
	}

	async fn handle_has_joined(&self, stream: &mut TcpStream, raw_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let query = raw_path.split('?').nth(1).unwrap_or("");
		let mut username = None;
		for pair in query.split('&') {
			let mut kv = pair.split('=');
			if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
				if k.eq_ignore_ascii_case("username") {
					username = Some(v.to_string());
					break;
				}
			}
		}

		if let Some(user_name) = username {
			let state = match State::get().await {
				Ok(s) => s,
				Err(_) => {
					let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";
					stream.write_all(resp.as_bytes()).await?;
					return Ok(());
				}
			};

			let mut matched_user = None;
			let users = crate::state::Credentials::get_all(&state.pool).await.unwrap_or_default();
			for entry in users.iter() {
				let creds = entry.value();
				if creds.is_offline() && creds.offline_profile.name.eq_ignore_ascii_case(&user_name) {
					matched_user = Some((creds.offline_profile.id, creds.offline_profile.name.clone()));
					break;
				}
			}

			if matched_user.is_none() {
				if let Ok(Some(active)) = crate::state::Credentials::get_active(&state.pool).await {
					if active.is_offline() && active.offline_profile.name.eq_ignore_ascii_case(&user_name) {
						matched_user = Some((active.offline_profile.id, active.offline_profile.name.clone()));
					}
				}
			}

			if let Some((user_id, name)) = matched_user {
				let uuid_str = user_id.as_simple().to_string();
				return self.send_profile_response(stream, &uuid_str, &name, user_id, &state).await;
			}
		}

		let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";
		stream.write_all(resp.as_bytes()).await?;
		Ok(())
	}

	async fn handle_profile(&self, stream: &mut TcpStream, raw_uuid: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let clean_uuid_str = raw_uuid.replace('-', "");
		let req_uuid = Uuid::parse_str(&clean_uuid_str).ok();

		let state = match State::get().await {
			Ok(s) => s,
			Err(_) => {
				let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";
				stream.write_all(resp.as_bytes()).await?;
				return Ok(());
			}
		};

		// 1. Check if uuid matches any offline user in database
		let mut matched_user: Option<(Uuid, String)> = None;
		if let Some(target_uuid) = req_uuid {
			let users = crate::state::Credentials::get_all(&state.pool).await.unwrap_or_default();
			for entry in users.iter() {
				let creds = entry.value();
				if creds.is_offline() {
					if creds.offline_profile.id == target_uuid
						|| creds.offline_profile.id.as_simple().to_string().eq_ignore_ascii_case(&clean_uuid_str)
						|| crate::state::offline_uuid(&creds.offline_profile.name) == target_uuid
					{
						matched_user = Some((creds.offline_profile.id, creds.offline_profile.name.clone()));
						break;
					}
				}
			}

			// If not matched yet, check active credential
			if matched_user.is_none() {
				if let Ok(Some(active)) = crate::state::Credentials::get_active(&state.pool).await {
					if active.is_offline() {
						if active.offline_profile.id == target_uuid
							|| active.offline_profile.id.as_simple().to_string().eq_ignore_ascii_case(&clean_uuid_str)
							|| crate::state::offline_uuid(&active.offline_profile.name) == target_uuid
						{
							matched_user = Some((active.offline_profile.id, active.offline_profile.name.clone()));
						}
					}
				}
			}

			// If still not matched, check offline_equipped_skins table
			if matched_user.is_none() {
				if let Ok(Some(_)) = sqlx::query("SELECT minecraft_user_uuid FROM offline_equipped_skins WHERE replace(minecraft_user_uuid, '-', '') = ?")
					.bind(&clean_uuid_str)
					.fetch_optional(&state.pool)
					.await
				{
					let user_name = sqlx::query_scalar::<_, String>("SELECT username FROM minecraft_users WHERE replace(uuid, '-', '') = ?")
						.bind(&clean_uuid_str)
						.fetch_optional(&state.pool)
						.await
						.ok()
						.flatten()
						.unwrap_or_else(|| "Player".to_string());
					matched_user = Some((target_uuid, user_name));
				}
			}
		}

		if let Some((user_id, username)) = matched_user {
			return self.send_profile_response(stream, &clean_uuid_str, &username, user_id, &state).await;
		}

		// 2. Proxy request for other players to Mojang
		let client = reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(4))
			.build()
			.unwrap_or_default();

		let mojang_url = format!(
			"https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false",
			clean_uuid_str
		);

		if let Ok(resp) = client.get(&mojang_url).send().await {
			if resp.status().is_success() {
				let body = resp.bytes().await.unwrap_or_default();
				let headers = format!(
					"HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
					body.len()
				);
				stream.write_all(headers.as_bytes()).await?;
				stream.write_all(&body).await?;
				return Ok(());
			}
		}

		// 3. Proxy to Ely.by if Mojang doesn't have it
		let ely_url = format!(
			"https://authserver.ely.by/api/authlib-injector/sessionserver/session/minecraft/profile/{}",
			clean_uuid_str
		);

		if let Ok(resp) = client.get(&ely_url).send().await {
			if resp.status().is_success() {
				let body = resp.bytes().await.unwrap_or_default();
				let headers = format!(
					"HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
					body.len()
				);
				stream.write_all(headers.as_bytes()).await?;
				stream.write_all(&body).await?;
				return Ok(());
			}
		}

		let resp = "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n";
		stream.write_all(resp.as_bytes()).await?;
		Ok(())
	}

	async fn handle_texture(&self, stream: &mut TcpStream, texture_key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
		let state = match State::get().await {
			Ok(s) => s,
			Err(_) => {
				let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
				stream.write_all(resp.as_bytes()).await?;
				return Ok(());
			}
		};

		// 1. Query custom_minecraft_skin_textures in SQLite dynamically
		let res = sqlx::query("SELECT texture FROM custom_minecraft_skin_textures WHERE texture_key = ?")
			.bind(texture_key)
			.fetch_optional(&state.pool)
			.await;

		if let Ok(Some(row)) = res {
			if let Ok(bytes) = row.try_get::<Vec<u8>, _>("texture") {
				if !bytes.is_empty() {
					return send_png(stream, &bytes).await;
				}
			}
		}

		// 2. Query offline_equipped_skins for URL (which could be data URL or HTTP URL)
		let stored_url = sqlx::query("SELECT url FROM offline_equipped_skins WHERE texture_key = ? LIMIT 1")
			.bind(texture_key)
			.fetch_optional(&state.pool)
			.await
			.ok()
			.flatten();

		if let Some(row) = stored_url {
			if let Ok(url_str) = row.try_get::<String, _>("url") {
				if let Some(base64_data) = url_str.strip_prefix("data:image/png;base64,") {
					if let Ok(bytes) = BASE64.decode(base64_data) {
						let _ = sqlx::query("INSERT OR REPLACE INTO custom_minecraft_skin_textures (texture_key, texture) VALUES (?, ?)")
							.bind(texture_key)
							.bind(&bytes)
							.execute(&state.pool)
							.await;
						return send_png(stream, &bytes).await;
					}
				} else if let Ok(url) = url::Url::parse(&url_str) {
					if let Some(bytes) = crate::api::minecraft_skins::png_util::data_url_to_bytes(&url) {
						let _ = sqlx::query("INSERT OR REPLACE INTO custom_minecraft_skin_textures (texture_key, texture) VALUES (?, ?)")
							.bind(texture_key)
							.bind(&bytes)
							.execute(&state.pool)
							.await;
						return send_png(stream, &bytes).await;
					} else if url_str.starts_with("http://") || url_str.starts_with("https://") {
						let client = reqwest::Client::builder()
							.timeout(std::time::Duration::from_secs(6))
							.build()
							.unwrap_or_default();
						if let Ok(resp) = client.get(&url_str).send().await {
							if resp.status().is_success() {
								if let Ok(bytes) = resp.bytes().await {
									let bytes_vec = bytes.to_vec();
									let _ = sqlx::query("INSERT OR REPLACE INTO custom_minecraft_skin_textures (texture_key, texture) VALUES (?, ?)")
										.bind(texture_key)
										.bind(&bytes_vec)
										.execute(&state.pool)
										.await;
									return send_png(stream, &bytes_vec).await;
								}
							}
						}
					}
				}
			}
		}

		// 3. If texture_key is a 64-char hex string (Mojang skin hash), fetch from Mojang textures
		if texture_key.len() == 64 && texture_key.chars().all(|c| c.is_ascii_hexdigit()) {
			let mojang_texture_url = format!("https://textures.minecraft.net/texture/{}", texture_key);
			let client = reqwest::Client::builder()
				.timeout(std::time::Duration::from_secs(6))
				.build()
				.unwrap_or_default();
			if let Ok(resp) = client.get(&mojang_texture_url).send().await {
				if resp.status().is_success() {
					if let Ok(bytes) = resp.bytes().await {
						let bytes_vec = bytes.to_vec();
						let _ = sqlx::query("INSERT OR REPLACE INTO custom_minecraft_skin_textures (texture_key, texture) VALUES (?, ?)")
							.bind(texture_key)
							.bind(&bytes_vec)
							.execute(&state.pool)
							.await;
						return send_png(stream, &bytes_vec).await;
					}
				}
			}
		}

		// 4. Fallback to bundled default skins
		for default_skin in crate::api::minecraft_skins::assets::DEFAULT_SKINS.iter() {
			if default_skin.texture_key.as_ref() == texture_key {
				if let Some(bytes) = crate::api::minecraft_skins::png_util::data_url_to_bytes(&default_skin.texture) {
					return send_png(stream, &bytes).await;
				}
			}
		}

		tracing::warn!(texture_key = texture_key, "Texture not found on offline skin server");
		let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
		stream.write_all(resp.as_bytes()).await?;
		Ok(())
	}
}

async fn send_png(stream: &mut TcpStream, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let headers = format!(
		"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
		bytes.len()
	);
	stream.write_all(headers.as_bytes()).await?;
	stream.write_all(bytes).await?;
	Ok(())
}

pub async fn ensure_tables() {
	if let Ok(state) = State::get().await {
		let _ = sqlx::query(
			"CREATE TABLE IF NOT EXISTS offline_equipped_skins (
				minecraft_user_uuid TEXT PRIMARY KEY,
				texture_key TEXT NOT NULL,
				variant TEXT NOT NULL,
				url TEXT NOT NULL
			)"
		)
		.execute(&state.pool)
		.await;
	}
}

/// Retrieves the equipped skin texture key and slim variant for an offline user.
async fn get_equipped_skin_for_offline_user(
	user_id: Uuid,
	_username: &str,
	state: &State,
) -> (String, bool) {
	// 1. Check offline equipped skins table if present
	let clean_uuid_str = user_id.as_simple().to_string();
	let stored = sqlx::query("SELECT texture_key, variant FROM offline_equipped_skins WHERE replace(minecraft_user_uuid, '-', '') = ?")
		.bind(&clean_uuid_str)
		.fetch_optional(&state.pool)
		.await
		.ok()
		.flatten();

	if let Some(row) = stored {
		let texture_key: String = row.try_get("texture_key").unwrap_or_default();
		let variant: String = row.try_get("variant").unwrap_or_default();
		let is_slim = variant.eq_ignore_ascii_case("SLIM");
		return (texture_key, is_slim);
	}

	// 2. Check in-memory profile cache
	if let Some(creds) = crate::state::Credentials::get_active(&state.pool).await.ok().flatten() {
		if creds.is_offline() && creds.offline_profile.id == user_id {
			if let Some(profile) = creds.online_profile().await {
				if let Ok(skin) = profile.current_skin() {
					let is_slim = matches!(skin.variant, crate::state::MinecraftSkinVariant::Slim);
					return (skin.texture_key().to_string(), is_slim);
				}
			}
		}
	}

	// 3. Fallback based on offline UUID parity (Classic Steve vs Slim Alex)
	let hash_parity = (user_id.as_u128() & 1) == 1;
	if hash_parity {
		// Alex (Slim)
		("1abc803022d8300ab7578b189294cce39622d9a404cdc00d3feacfdf45be6981".to_string(), true)
	} else {
		// Steve (Classic)
		("31f477eb1a7beee631c2ca64d06f8f68fa93a3386d04452ab27f43acdf1b60cb".to_string(), false)
	}
}

/// Ensures authlib-injector.jar exists in the libraries folder, downloading it if necessary.
pub async fn ensure_authlib_injector(directories: &DirectoryInfo) -> crate::Result<PathBuf> {
	let target_dir = directories.libraries_dir();
	tokio::fs::create_dir_all(&target_dir).await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to create libraries directory: {e}"))
	})?;

	let target_path = target_dir.join("authlib-injector.jar");
	if target_path.exists() {
		if let Ok(metadata) = tokio::fs::metadata(&target_path).await {
			if metadata.len() > 100_000 {
				return Ok(target_path);
			}
		}
	}

	let client = reqwest::Client::builder()
		.timeout(std::time::Duration::from_secs(20))
		.build()
		.map_err(|e| crate::ErrorKind::LauncherError(format!("Failed to build reqwest client: {e}")))?;

	tracing::info!("Downloading authlib-injector.jar from {}", AUTHLIB_INJECTOR_URL_PRIMARY);
	let res = client.get(AUTHLIB_INJECTOR_URL_PRIMARY).send().await;
	let bytes = match res {
		Ok(resp) if resp.status().is_success() => resp.bytes().await.map_err(|e| {
			crate::ErrorKind::LauncherError(format!("Failed to read authlib-injector body: {e}"))
		})?,
		_ => {
			tracing::warn!("Primary authlib-injector download failed, attempting mirror {}", AUTHLIB_INJECTOR_URL_MIRROR);
			let mirror_res = client
				.get(AUTHLIB_INJECTOR_URL_MIRROR)
				.send()
				.await
				.map_err(|e| crate::ErrorKind::LauncherError(format!("Mirror download failed: {e}")))?;
			mirror_res.bytes().await.map_err(|e| {
				crate::ErrorKind::LauncherError(format!("Failed to read mirror authlib-injector body: {e}"))
			})?
		}
	};

	tokio::fs::write(&target_path, bytes).await.map_err(|e| {
		crate::ErrorKind::LauncherError(format!("Failed to write authlib-injector.jar: {e}"))
	})?;

	tracing::info!("Successfully saved authlib-injector.jar to {}", target_path.display());
	Ok(target_path)
}
