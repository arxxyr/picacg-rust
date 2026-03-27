//! WebSocket 连接管理器
//!
//! 使用 tokio-tungstenite 实现异步 WebSocket 连接，
//! 通过 mpsc 通道与 Bevy ECS 主线程通信。

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message as WsMessage;

/// 启动 WebSocket 连接
///
/// 返回值：
/// - `incoming_rx`: 接收来自服务器的消息（JSON 文本）
/// - `outgoing_tx`: 发送消息到服务器（JSON 文本）
/// - `close_tx`: 发送关闭信号
pub async fn connect_websocket(
    url: &str,
) -> Result<
    (
        mpsc::UnboundedReceiver<String>,
        mpsc::UnboundedSender<String>,
        oneshot::Sender<()>,
    ),
    String,
> {
    tracing::info!("正在连接 WebSocket: {}", url);

    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| format!("WebSocket 连接失败: {}", e))?;

    tracing::info!("WebSocket 连接成功");

    let (mut write, mut read) = ws_stream.split();

    // 服务器 -> 客户端 的消息通道
    let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<String>();
    // 客户端 -> 服务器 的消息通道
    let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel::<String>();
    // 关闭信号
    let (close_tx, mut close_rx) = oneshot::channel::<()>();

    // 读取任务：从 WebSocket 读取消息并转发到 incoming_tx
    let incoming_tx_clone = incoming_tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text)))
                            if incoming_tx_clone.send(text.to_string()).is_err() =>
                        {
                            tracing::debug!("WebSocket incoming 通道已关闭");
                            break;
                        }
                        Some(Ok(WsMessage::Text(_))) => {}
                        Some(Ok(WsMessage::Ping(data))) => {
                            tracing::debug!("收到 WebSocket Ping");
                            // Pong 由 tungstenite 自动处理
                            let _ = data;
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            tracing::info!("WebSocket 服务器关闭连接");
                            break;
                        }
                        Some(Err(e)) => {
                            tracing::error!("WebSocket 读取错误: {}", e);
                            break;
                        }
                        None => {
                            tracing::info!("WebSocket 流结束");
                            break;
                        }
                        _ => {}
                    }
                }
                _ = &mut close_rx => {
                    tracing::info!("收到 WebSocket 关闭信号");
                    break;
                }
            }
        }
        tracing::info!("WebSocket 读取任务退出");
    });

    // 写入任务：从 outgoing_rx 读取消息并发送到 WebSocket
    tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if let Err(e) = write.send(WsMessage::Text(msg.into())).await {
                tracing::error!("WebSocket 发送错误: {}", e);
                break;
            }
        }
        // 尝试优雅关闭
        let _ = write.close().await;
        tracing::info!("WebSocket 写入任务退出");
    });

    Ok((incoming_rx, outgoing_tx, close_tx))
}
