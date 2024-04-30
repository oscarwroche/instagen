use crate::{
    constants::{FB_APP_ID, FB_OAUTH_REDIRECT_URL, IG_POST_CAPTION, IG_USER_ID, TCP_LISTENER_IP},
    utils::{extract_query_parameter, extract_s3_file_uri_from_state},
};
use reqwest::Client;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use serde::Deserialize;
use std::{
    env, error,
    fs::File,
    io::{self, BufReader},
    path::Path,
    sync::Arc,
};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    spawn,
};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
}

#[derive(Deserialize)]
struct InstagramMediaContainer {
    id: String,
}

fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

fn load_keys(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .next()
        .unwrap()
        .map(Into::into)
}

pub async fn spawn_server() -> tokio::task::JoinHandle<()> {
    spawn(async move {
        let args: Vec<String> = env::args().collect();

        let certs = load_certs(Path::new(&args[1])).unwrap();
        let key = load_keys(Path::new(&args[2])).unwrap();

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));

        let listener = TcpListener::bind(TCP_LISTENER_IP).await.unwrap();
        println!("Listening on: {}", TCP_LISTENER_IP);

        // Accept connections
        let (socket, _) = listener.accept().await.unwrap();
        let accepted_stream = acceptor.accept(socket).await.unwrap();

        let (code, state) = handle_client_request(accepted_stream).await.unwrap();

        let access_token = get_access_token(code).await.unwrap();

        let is_debug = args.get(3);

        match is_debug {
            Some(_) => println!("Debug mode: image post skipped"),
            None => {
                let s3_file_uri = extract_s3_file_uri_from_state(state).unwrap();
                post_picture(s3_file_uri, access_token).await.unwrap()
            }
        }
    })
}

async fn handle_client_request(
    mut accepted_stream: TlsStream<TcpStream>,
) -> Result<(String, String), Box<dyn error::Error>> {
    let mut buf = Vec::new();
    let mut temp_buf = [0; 1024]; // Temporary buffer to read chunks
    loop {
        let n = accepted_stream.read(&mut temp_buf).await?;
        if n == 0 {
            break;
        } // End of stream

        buf.extend_from_slice(&temp_buf[..n]);

        // Check if we've reached the end of the headers
        if buf.windows(4).any(|window| window == b"\r\n\r\n") {
            let request = String::from_utf8_lossy(&buf);
            println!("Received request:\n{}", request);
            // Assuming the URL is properly encoded and parameters are well-formed
            let code = extract_query_parameter(&request, "code");
            let state = extract_query_parameter(&request, "state");

            match (code, state) {
                (Some(code_value), Some(state_value)) => {
                    println!("Code: {}", code_value);
                    println!("State: {}", state_value);
                    return Ok((code_value.to_string(), state_value.to_string()));
                }
                _ => break,
            }
        }
    }
    Err("Failed to parse the code from the user request".into())
}

async fn get_access_token(code: String) -> Result<String, Box<dyn error::Error>> {
    let app_id = FB_APP_ID;
    let redirect_uri = format!("{}/", FB_OAUTH_REDIRECT_URL);

    let client_secret = env::var("FB_CLIENT_SECRET")?;

    let url = format!(
            "https://graph.facebook.com/v19.0/oauth/access_token?client_id={}&redirect_uri={}&client_secret={}&code={}",
            app_id, redirect_uri, client_secret, code
        );

    let response = Client::new().get(url).send().await?;

    // Ensure the request was successful (status code 2xx)
    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()).into());
    }

    // Read the response body as a string
    let access_token = response.json::<AccessToken>().await?.access_token;
    println!("Access token: {}", access_token);

    Ok(access_token)
}

async fn post_picture(
    s3_file_uri: String,
    access_token: String,
) -> Result<(), Box<dyn error::Error>> {
    let ig_user_id = IG_USER_ID;
    let caption = IG_POST_CAPTION;

    let media_request_url = format!(
        "https://graph.facebook.com/v19.0/{}/media?image_url={}&access_token={}&caption={}",
        ig_user_id, s3_file_uri, access_token, caption
    );

    let container_id_response = Client::new().post(media_request_url).send().await?;

    // Ensure the request was successful (status code 2xx)
    if !container_id_response.status().is_success() {
        return Err(format!("Request failed: {}", container_id_response.text().await?).into());
    }

    let container_id = container_id_response
        .json::<InstagramMediaContainer>()
        .await?
        .id;

    let media_publish_request_url = format!(
        "https://graph.facebook.com/v19.0/{}/media_publish?creation_id={}&access_token={}",
        ig_user_id, container_id, access_token,
    );

    Client::new().post(media_publish_request_url).send().await?;

    Ok(())
}
