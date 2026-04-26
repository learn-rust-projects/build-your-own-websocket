use std::collections::BTreeMap;

use anyhow::{self, Result, bail};
use base64::{Engine, engine::general_purpose};
use ring::digest;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
#[tokio::main]
async fn main() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut writer = BufWriter::new(write_half);
        handshake(&mut reader, &mut writer).await?;
    }

    Ok(())
}

async fn handshake<R, W>(reader: &mut R, writer: &mut W) -> Result<()>
where
    R: AsyncBufRead + std::marker::Unpin,
    W: AsyncWrite + std::marker::Unpin,
{
    let mut buffer = String::new();
    let mut headers = BTreeMap::<String, String>::new();

    // 1. 读取请求行（优化：直接丢弃，不做多余操作）
    reader.read_line(&mut buffer).await?;
    buffer.clear();

    // 2. 读取请求头
    loop {
        // 每次读取前清空，避免追加
        buffer.clear();

        let n = reader.read_line(&mut buffer).await?;

        if n == 0 {
            bail!("unexpected eof");
        }
        let header_line = &buffer;
        // 头信息完结
        if header_line == "\r\n" {
            break;
        }
        // 安全去掉 \r\n，不会 panic！
        let line = header_line.strip_suffix("\r\n").unwrap_or(header_line);

        // 解析头
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.to_lowercase(), v.trim_start().into());
        };

        buffer.clear();
    }
    // 3. 获取 WebSocket Key
    let sec_websocket_key = headers
        .get("sec-websocket-key")
        .ok_or_else(|| anyhow::anyhow!("no header Sec-Websocket-Key"))?;

    // 4. 计算响应
    const UUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concat_str = [sec_websocket_key.as_bytes(), UUID].concat();
    let hash_result = digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, &concat_str);
    let sec_websocket_accept = general_purpose::STANDARD.encode(hash_result.as_ref());

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        sec_websocket_accept
    );

    writer.write_all(response.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn smoke_test() {
        assert!(std::env::args().next().is_some());
    }
}
