use std::collections::BTreeMap;

use anyhow::{self, Result, bail};
use base64::{Engine, engine::general_purpose};
use bytes::{BufMut, BytesMut};
use ring::digest;
use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter,
};
#[tokio::main]
async fn main() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("server is running on 0.0.0.0:8080");
    while let Ok((stream, _)) = listener.accept().await {
        println!("new connection");
        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut writer = BufWriter::new(write_half);
        handshake(&mut reader, &mut writer).await?;
        send_messages_to_client(&mut reader, &mut writer).await?;
        receive_messages_from_client(&mut reader, &mut writer).await?;
    }
    println!("server is closed");

    Ok(())
}
async fn send_messages_to_client<R, W>(_reader: &mut R, writer: &mut W) -> Result<()>
where
    R: AsyncBufRead + std::marker::Unpin,
    W: AsyncWrite + std::marker::Unpin,
{
    let message = Message::Text("hello from server side".into());
    writer.write_all(&message.encode()).await?;

    let middle_message = Message::Text(String::from_utf8_lossy(&[b'*'; 126]).to_string());
    writer.write_all(&middle_message.encode()).await?;

    let big_message =
        Message::Text(String::from_utf8_lossy(&[b'*'; (u16::MAX as usize + 1)]).to_string());
    writer.write_all(&big_message.encode()).await?;

    let big_binary_message = Message::Binary(vec![b'*'; u16::MAX as usize + 1]);
    writer.write_all(&big_binary_message.encode()).await?;
    writer.flush().await?;
    Ok(())
}

async fn receive_messages_from_client<R, W>(reader: &mut R, _writer: &mut W) -> Result<()>
where
    R: AsyncBufRead + std::marker::Unpin,
    W: AsyncWrite + std::marker::Unpin,
{
    while let Ok(message) = Message::decode(reader).await {
        match message {
            Message::Text(text) => {
                println!("{text}");
            }
            Message::Binary(data) => {
                println!("{data:?}")
            }
        }
    }
    Ok(())
}

enum Message {
    Text(String),    // 文本消息
    Binary(Vec<u8>), // 二进制消息
}

/// 消息的编码与解码
impl Message {
    fn as_bytes(&self) -> &[u8] {
        match &self {
            Message::Binary(data) => data,
            Message::Text(data) => data.as_bytes(),
        }
    }

    // opcode  表示当前的消息类型
    fn opcode(&self) -> u8 {
        match &self {
            Message::Binary(_) => 2,
            Message::Text(_) => 1,
        }
    }
    fn encode(&self) -> Vec<u8> {
        let payload_data = self.as_bytes();
        let payload_length = payload_data.len() as u64;

        let mut total_frame_length = 2;
        if payload_length > 125 {
            if payload_length > u16::MAX as u64 {
                total_frame_length += 8;
            } else {
                total_frame_length += 2;
            }
        }
        total_frame_length += payload_length;

        let mut frame = BytesMut::with_capacity(total_frame_length as usize);
        frame.put_u8(0b1000_0000 | self.opcode()); // fin 是 1
        if payload_length <= 125 {
            frame.put_u8(payload_length as u8);
        } else if payload_length > u16::MAX as u64 {
            frame.put_u8(127);
            frame.extend_from_slice(&payload_length.to_be_bytes());
        } else {
            frame.put_u8(126);
            frame.extend_from_slice(&(payload_length as u16).to_be_bytes());
        }
        frame.extend_from_slice(payload_data);
        frame.to_vec()
    }

    async fn decode<R: AsyncBufRead + std::marker::Unpin>(reader: &mut R) -> Result<Self> {
        // 1. 读取固定 2 字节头部
        let mut header = [0u8; 2];
        reader.read_exact(&mut header).await?;

        // 不考虑 fin 不为 1 的情况，一次读取一个 frame 然后拼接成 message
        let opcode = header[0] & 0b1111;
        let mask = header[1] >> 7;
        if mask != 1 {
            bail!("mask required");
        }
        let mut payload_len = (header[1] & 0b0111_1111) as u64;

        // 2. 处理扩展长度
        if payload_len == 126 {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf).await?;
            payload_len = u16::from_be_bytes(buf) as u64;
        } else if payload_len == 127 {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf).await?;
            payload_len = u64::from_be_bytes(buf);
        }
        let mut mask_key = [0; 4];
        reader.read_exact(&mut mask_key).await?;

        let mut payload_data: Vec<u8> = vec![0; payload_len as usize];
        reader.read_exact(&mut payload_data).await?;
        (0..payload_data.len()).for_each(|i| {
            let j = i % 4;
            let cur_mask_key = mask_key[j];
            payload_data[i] ^= cur_mask_key;
        });

        Ok(if opcode == 1 {
            Message::Text(String::from_utf8_lossy(&payload_data).to_string())
        } else {
            Message::Binary(payload_data)
        })
    }
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
