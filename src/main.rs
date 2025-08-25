mod parse;

use bark_rs::{AsyncBarkClient, Level};
use bollard::{Docker, container::LogOutput, query_parameters::LogsOptions};
use clap::Parser;
use eros::{IntoDynTracedError, Result};
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::sync::LazyLock;
use tracing::{error, info, warn};

#[derive(Parser)]
struct Args {
    #[arg(short('n'), long, env)]
    container_name: String,
    #[arg(short('k'), long, env)]
    bark_key: String,
}
static ARGS: LazyLock<Args> = LazyLock::new(|| Args::parse());
static BARK_CLIENT: LazyLock<AsyncBarkClient> =
    LazyLock::new(|| AsyncBarkClient::with_device_key("https://api.day.app", &ARGS.bark_key));

fn main() {
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let docker = Docker::connect_with_local_defaults().unwrap();
        let container_name = &ARGS.container_name;

        let mut stream = docker.logs(
            container_name,
            Some(LogsOptions {
                follow: true,
                stdout: true,
                stderr: true,
                tail: "0".into(), // 关键：不回放历史，仅从当前会话往后
                ..Default::default()
            }),
        );

        while let Some(item) = stream.next().await {
            match item {
                Ok(log) => match log {
                    LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
                        tokio::spawn(async move {
                            let message = String::from_utf8_lossy(&message);
                            if let Err(err) = handle_message(&message).await {
                                error!("Error: {}, {}", err, message);
                            };
                        });
                    }
                    _ => {}
                },
                Err(err) => error!("Error during log streaming: {}", err),
            }
        }
    });
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct RecorgMessage {
    time: String,
    level: String,
    msg: String,
    host: String,
    room: String,
}

async fn handle_message(message: &str) -> Result<()> {
    let message = parse::parse(message);

    let rm: Result<_> = (|| {
        Ok(serde_value::to_value(message)
            .traced_dyn()?
            .deserialize_into::<RecorgMessage>()
            .traced_dyn()?)
    })();

    let rm = match rm {
        Ok(rm) => rm,
        Err(err) => {
            warn!("Warn: {}", err);
            return Ok(());
        }
    };

    match rm.msg.as_str() {
        "Live Start" => {
            info!("Live Start: {}", rm.host);
            BARK_CLIENT
                .message()
                .title(&format!("Live Start: {}", rm.host))
                .send()
                .await
                .traced_dyn()?;
        }
        "<nil>" => {
            info!("Live End: {}", rm.host);
            BARK_CLIENT
                .message()
                .title(&format!("Live End: {}", rm.host))
                .send()
                .await
                .traced_dyn()?;
        }
        _ => {
            info!("Unknown message: {}, skip", rm.msg);
        }
    }

    Ok(())
}
