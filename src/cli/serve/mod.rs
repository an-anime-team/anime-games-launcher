use std::path::{Path, PathBuf};

use axum::{body::Body, http::{Request, Response, StatusCode}, response::IntoResponse, routing::get, Router};

mod config;

pub async fn run(dir_to_serve: String) -> anyhow::Result<()> {
    let current_path = PathBuf::from(dir_to_serve);
    let config_file_path = current_path.join(config::CONFIG_FILE_NAME);

    let config: config::Config = match tokio::fs::read_to_string(config_file_path).await {
        Ok(content) => match toml::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    "Failed to read config file, using default one, please check your config : {}",
                    e
                );
                config::Config::default()
            }
        },
        Err(_) => {
            tracing::warn!(
                "Seems that your config file is missing, creating default configuration for you...."
            );
            let config = config::Config::default();
            let config_path = current_path.join(config::CONFIG_FILE_NAME);
            let new_config_file_content = toml::to_string(&config)?;

            tokio::fs::write(config_path, new_config_file_content.as_bytes()).await?;

            config
        }
    };

    let ip = config.serve_at.ip.parse::<std::net::Ipv4Addr>()?;
    let port = config.serve_at.port;

    let addr = std::net::SocketAddr::from((ip, port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let dir_to_serve = current_path.clone();
    let router = Router::new().fallback_service(get(move |req: Request<Body>| {
        serve_dir(req, dir_to_serve, config.mislead)
    }));

    tracing::info!("listening on \t-> {}", listener.local_addr()?);
    tracing::info!("serving dir \t-> {}", current_path.to_str().unwrap());
    axum::serve(listener, router).await?;

    Ok(())
}

async fn serve_dir(
    request: Request<Body>,
    base_dir: PathBuf,
    mislead: config::Mislead,
) -> impl IntoResponse {
    let uri_path = request.uri().path().trim_start_matches("/");
    let file_path = base_dir.join(uri_path);

    if !file_path.exists() || file_path.is_dir() {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }

    tracing::info!("Serving \t-> {}", file_path.to_str().unwrap());

    if let Some(mime_type) = is_image_file(file_path.clone()).await.unwrap_or(None) {
        match tokio::fs::read(file_path).await {
            Ok(content) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type)
                .body(Body::from(content))
                .unwrap(),
            Err(err) => {
                tracing::error!("Failed to read file : {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Error reading file").into_response()
            }
        }
    } else {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                let edited_content = content.replace(&mislead.link_to_mislead, &mislead.mislead_to);

                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-type", "application/octet-stream")
                    .body(Body::from(edited_content))
                    .unwrap()
            }
            Err(err) => {
                tracing::error!("Failed to read file : {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Error reading file").into_response()
            }
        }
    }
}

async fn is_image_file(path: impl AsRef<Path>) -> anyhow::Result<Option<String>> {
    let kind = infer::get_from_path(path)?;

    if let Some(file) = kind {
        if file.mime_type().starts_with("image") {
            Ok(Some(String::from(file.mime_type())))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
