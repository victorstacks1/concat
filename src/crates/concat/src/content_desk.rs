use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::Read as IoRead;
use std::path::Path;

const ROUTE: &str = "/api/integrations/concat/reels";
const MAX_CHUNK_SIZE: u64 = 32 * 1024 * 1024;
const MAX_VIDEO_SIZE: u64 = 300_000_000;

/// A Reel card the Content Desk allows Concat to publish.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ContentDeskCard {
    /// The Content Desk production card ID.
    pub id: String,
    /// The title shown in the Content Desk board.
    pub title: String,
}

/// How the Content Desk schedules a Reel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueMode {
    /// Queue into the near-term slot.
    Now,
    /// Queue into the next configured strategy slot.
    Strategy,
}

impl QueueMode {
    /// The value sent in the Content Desk API request.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Now => "now",
            Self::Strategy => "strategy",
        }
    }
}

/// The result returned after the Content Desk has queued a Reel.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct QueuedPublication {
    /// The card which was queued.
    pub card_id: String,
    /// The card's title.
    pub title: String,
    /// The selected scheduling mode.
    pub mode: String,
    /// Always `queued`; Buffer publication happens later.
    pub status: String,
    /// The scheduled publication date and time.
    pub data_postar: String,
}

/// A safe error from the Content Desk integration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentDeskError(String);

impl ContentDeskError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ContentDeskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ContentDeskError {}

/// Builds the integration route after trimming a custom Site URL.
pub fn endpoint(base_url: &str) -> Result<String, ContentDeskError> {
    let base = base_url.trim().trim_end_matches('/');
    if !(base.starts_with("https://") || base.starts_with("http://"))
        || base.bytes().any(|byte| byte.is_ascii_whitespace())
        || base.contains('?')
        || base.contains('#')
    {
        return Err(ContentDeskError::new(
            "Informe uma URL HTTP ou HTTPS válida do Content Desk.",
        ));
    }
    Ok(format!("{base}{ROUTE}"))
}

fn response_text(response: ureq::Response) -> Result<String, ContentDeskError> {
    response
        .into_string()
        .map_err(|_| ContentDeskError::new("Não foi possível ler a resposta do Content Desk."))
}

fn send(request: ureq::Request, body: Option<&[u8]>) -> Result<ureq::Response, ContentDeskError> {
    let result = match body {
        Some(bytes) => request.send_bytes(bytes),
        None => request.call(),
    };
    result.map_err(|error| match error {
        ureq::Error::Status(status, _) => {
            ContentDeskError::new(format!("Content Desk respondeu HTTP {status}."))
        }
        ureq::Error::Transport(_) => {
            ContentDeskError::new("Não foi possível conectar ao Content Desk.")
        }
    })
}

fn get_json<T: for<'de> Deserialize<'de>>(url: &str, api_key: &str) -> Result<T, ContentDeskError> {
    let response = send(
        ureq::get(url)
            .set("x-api-key", api_key)
            .set("Accept", "application/json"),
        None,
    )?;
    serde_json::from_str(&response_text(response)?)
        .map_err(|_| ContentDeskError::new("O Content Desk retornou uma resposta inválida."))
}

fn post_json<T: for<'de> Deserialize<'de>, B: Serialize>(
    url: &str,
    api_key: &str,
    action: &str,
    body: &B,
) -> Result<T, ContentDeskError> {
    let encoded = serde_json::to_vec(body).map_err(|_| {
        ContentDeskError::new("Não foi possível preparar a solicitação ao Content Desk.")
    })?;
    let response = send(
        ureq::post(url)
            .set("x-api-key", api_key)
            .set("x-reels-upload-action", action)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json"),
        Some(&encoded),
    )?;
    serde_json::from_str(&response_text(response)?)
        .map_err(|_| ContentDeskError::new("O Content Desk retornou uma resposta inválida."))
}

/// Fetches the API key owner's Reels that are in `Gravado`.
pub fn list_cards(base_url: &str, api_key: &str) -> Result<Vec<ContentDeskCard>, ContentDeskError> {
    if api_key.trim().is_empty() {
        return Err(ContentDeskError::new(
            "Configure a chave de API do Content Desk nas preferências.",
        ));
    }
    let api_key = api_key.trim();
    #[derive(Deserialize)]
    struct CardsResponse {
        cards: Vec<ContentDeskCard>,
    }
    let response: CardsResponse = get_json(&endpoint(base_url)?, api_key)?;
    Ok(response.cards)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartUpload<'a> {
    production_id: &'a str,
    name: &'a str,
    size_bytes: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartedUpload {
    upload_token: String,
    chunk_size: u64,
}

#[derive(Deserialize)]
struct UploadedPart {
    #[serde(rename = "partNumber")]
    part_number: u32,
    etag: String,
}

#[derive(Serialize)]
struct PublishUpload<'a> {
    action: &'static str,
    production_id: &'a str,
    mode: &'a str,
    #[serde(rename = "uploadToken")]
    upload_token: &'a str,
    parts: &'a [UploadedPartBody],
}

#[derive(Serialize)]
struct UploadedPartBody {
    #[serde(rename = "partNumber")]
    part_number: u32,
    etag: String,
}

#[derive(Serialize)]
struct AbortUpload<'a> {
    #[serde(rename = "uploadToken")]
    upload_token: &'a str,
}

fn upload_part(
    url: &str,
    api_key: &str,
    upload_token: &str,
    part_number: u32,
    bytes: &[u8],
) -> Result<UploadedPart, ContentDeskError> {
    #[derive(Deserialize)]
    struct PartResponse {
        #[serde(rename = "partNumber")]
        part_number: u32,
        etag: String,
    }
    let response = send(
        ureq::post(url)
            .set("x-api-key", api_key)
            .set("x-reels-upload-action", "part")
            .set("x-reels-upload-token", upload_token)
            .set("x-reels-part-number", &part_number.to_string())
            .set("Content-Type", "application/octet-stream")
            .set("Content-Length", &bytes.len().to_string()),
        Some(bytes),
    )?;
    let parsed: PartResponse = serde_json::from_str(&response_text(response)?).map_err(|_| {
        ContentDeskError::new("O Content Desk retornou uma confirmação inválida para uma parte.")
    })?;
    if parsed.part_number != part_number || parsed.etag.is_empty() {
        return Err(ContentDeskError::new(
            "O Content Desk confirmou uma parte inválida do vídeo.",
        ));
    }
    Ok(UploadedPart {
        part_number: parsed.part_number,
        etag: parsed.etag,
    })
}

/// Uploads a rendered MP4 and queues the chosen card in the Content Desk.
/// `progress` receives the number of bytes uploaded and the file's total size.
pub fn upload_and_queue(
    base_url: &str,
    api_key: &str,
    production_id: &str,
    mode: QueueMode,
    path: &Path,
    mut progress: impl FnMut(u64, u64) -> bool,
) -> Result<QueuedPublication, ContentDeskError> {
    if api_key.trim().is_empty() {
        return Err(ContentDeskError::new(
            "Configure a chave de API do Content Desk nas preferências.",
        ));
    }
    let api_key = api_key.trim();
    let url = endpoint(base_url)?;
    let metadata = std::fs::metadata(path)
        .map_err(|_| ContentDeskError::new("Não foi possível abrir o vídeo exportado."))?;
    let total = metadata.len();
    if total == 0 || total >= MAX_VIDEO_SIZE {
        return Err(ContentDeskError::new(
            "O vídeo precisa ter mais de 0 e menos de 300 MB.",
        ));
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("concat-export.mp4");
    let started: StartedUpload = post_json(
        &url,
        api_key,
        "start",
        &StartUpload {
            production_id,
            name,
            size_bytes: total,
        },
    )?;
    if started.upload_token.is_empty()
        || started.chunk_size == 0
        || started.chunk_size > MAX_CHUNK_SIZE
    {
        let _ = post_json::<serde_json::Value, _>(
            &url,
            api_key,
            "abort",
            &AbortUpload {
                upload_token: &started.upload_token,
            },
        );
        return Err(ContentDeskError::new(
            "O Content Desk retornou uma sessão de upload inválida.",
        ));
    }

    let result = (|| {
        let mut file = File::open(path)
            .map_err(|_| ContentDeskError::new("Não foi possível abrir o vídeo exportado."))?;
        let mut parts = Vec::new();
        let mut uploaded = 0;
        let mut part_number = 1u32;
        while uploaded < total {
            let chunk_len = (total - uploaded).min(started.chunk_size) as usize;
            let mut chunk = vec![0; chunk_len];
            file.read_exact(&mut chunk)
                .map_err(|_| ContentDeskError::new("Não foi possível ler o vídeo exportado."))?;
            let part = upload_part(&url, api_key, &started.upload_token, part_number, &chunk)?;
            uploaded += chunk_len as u64;
            parts.push(UploadedPartBody {
                part_number: part.part_number,
                etag: part.etag,
            });
            if progress(uploaded, total) {
                return Err(ContentDeskError::new("Envio cancelado."));
            }
            part_number += 1;
        }
        let queued: QueuedPublication = post_json(
            &url,
            api_key,
            "publish",
            &PublishUpload {
                action: "publish",
                production_id,
                mode: mode.as_str(),
                upload_token: &started.upload_token,
                parts: &parts,
            },
        )?;
        if queued.status != "queued"
            || queued.card_id != production_id
            || queued.mode != mode.as_str()
            || queued.data_postar.is_empty()
        {
            return Err(ContentDeskError::new(
                "A resposta do Content Desk não confirmou a inclusão na fila.",
            ));
        }
        Ok(queued)
    })();
    if result.is_err() {
        let _ = post_json::<serde_json::Value, _>(
            &url,
            api_key,
            "abort",
            &AbortUpload {
                upload_token: &started.upload_token,
            },
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn serve(status: &str, body: &str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let body = body.to_owned();
        let status = status.to_owned();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let size = stream.read(&mut request).unwrap();
            let headers = String::from_utf8_lossy(&request[..size]).to_string();
            write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            headers
        });
        (format!("http://{address}"), server)
    }

    #[test]
    fn normalizes_the_site_url_before_appending_the_route() {
        assert_eq!(
            endpoint(" https://desk.example/// ").unwrap(),
            "https://desk.example/api/integrations/concat/reels"
        );
        assert!(endpoint("not a URL").is_err());
    }

    #[test]
    fn lists_only_card_ids_and_titles_and_sends_the_api_key_header() {
        let (base, server) = serve(
            "200 OK",
            r#"{"cards":[{"id":"a","title":"Reel A","status":"ignored"}]}"#,
        );
        let cards = list_cards(&base, "private-test-key").unwrap();
        let request = server.join().unwrap().to_lowercase();
        assert!(
            request.starts_with("get /api/integrations/concat/reels "),
            "{request}"
        );
        assert!(request.contains("x-api-key: private-test-key"), "{request}");
        assert_eq!(
            cards,
            vec![ContentDeskCard {
                id: "a".into(),
                title: "Reel A".into()
            }]
        );
    }

    #[test]
    fn reports_http_errors_without_returning_credentials() {
        let (base, server) = serve("401 Unauthorized", r#"{"error":"Chave inválida"}"#);
        let error = list_cards(&base, "private-test-key")
            .unwrap_err()
            .to_string();
        let _ = server.join().unwrap();
        assert!(error.contains("401"), "{error}");
        assert!(!error.contains("private-test-key"), "{error}");
    }

    #[test]
    fn rejects_malformed_card_json() {
        let (base, server) = serve("200 OK", "not-json");
        assert!(list_cards(&base, "key").is_err());
        let _ = server.join().unwrap();
    }
}
