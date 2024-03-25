pub mod server {
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
        sync::mpsc::Sender,
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

    pub async fn spawn_server(tx: Sender<String>) -> tokio::task::JoinHandle<()> {
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

            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            println!("Listening on: 127.0.0.1:8080");

            // Accept connections
            let (socket, _) = listener.accept().await.unwrap();
            let accepted_stream = acceptor.accept(socket).await.unwrap();

            let code = handle_client_request(accepted_stream).await.unwrap();

            let access_token = get_access_token(code).await.unwrap();

            post_picture(access_token).await.unwrap();

            if let Err(e) = tx.send("Success".to_string()).await {
                eprintln!("Failed to send message to main thread: {}", e);
            };
        })
    }

    async fn handle_client_request(
        mut accepted_stream: TlsStream<TcpStream>,
    ) -> Result<String, Box<dyn error::Error>> {
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
                // Process the request here
                if let Some(code_start) = request.find("/?code=") {
                    // Start index is after "/?code="
                    let start_index = code_start + "/?code=".len();
                    // End index is either the next "&" or the end of the line
                    let end_index = request[start_index..]
                        .find('&')
                        .map_or_else(|| request[start_index..].find(' ').unwrap_or(0), |v| v)
                        + start_index;

                    let code_value = &request[start_index..end_index];
                    println!("Code: {}", code_value);
                    return Ok(code_value.to_string());
                }
                break;
            }
        }
        Err("Failed to parse the code from the user request".into())
    }

    async fn get_access_token(code: String) -> Result<String, Box<dyn error::Error>> {
        let app_id = "986335749574127";
        let redirect_uri = "https://127.0.0.1:8080/";

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

    async fn post_picture(access_token: String) -> Result<(), Box<dyn error::Error>> {
        let image_url = "https://miro.medium.com/v2/resize:fit:1024/0*wATbQ49jziZTyhZH.jpg";
        let ig_user_id = "17841464829741641";

        let media_request_url = format!(
            "https://graph.facebook.com/v19.0/{}/media?image_url={}&access_token={}",
            ig_user_id, image_url, access_token,
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
}
