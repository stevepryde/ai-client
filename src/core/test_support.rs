use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub(crate) struct RedirectServer {
    pub base_url: String,
    pub origin_request: tokio::task::JoinHandle<String>,
    pub redirect_target: tokio::net::TcpListener,
}

pub(crate) async fn one_shot_server(
    path_prefix: &str,
    response: String,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 16 * 1024];
        let read = socket.read(&mut request).await.unwrap();
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&request[..read]).into_owned()
    });
    (
        format!("http://{address}/{}", path_prefix.trim_matches('/')),
        handle,
    )
}

pub(crate) async fn delayed_server(
    path_prefix: &str,
    response: String,
    delay: std::time::Duration,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 4096];
        let _ = socket.read(&mut request).await;
        tokio::time::sleep(delay).await;
        let _ = socket.write_all(response.as_bytes()).await;
    });
    (
        format!("http://{address}/{}", path_prefix.trim_matches('/')),
        handle,
    )
}

pub(crate) async fn cross_origin_redirect_server(path_prefix: &str) -> RedirectServer {
    let redirect_target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_address = redirect_target.local_addr().unwrap();
    let response = format!(
        "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/steal\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let (base_url, origin_request) = one_shot_server(path_prefix, response).await;
    RedirectServer {
        base_url,
        origin_request,
        redirect_target,
    }
}

pub(crate) async fn chunked_server(
    chunks: Vec<Vec<u8>>,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 4096];
        let read = socket.read(&mut request).await.unwrap();
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
        for chunk in chunks {
            socket
                .write_all(format!("{:X}\r\n", chunk.len()).as_bytes())
                .await
                .unwrap();
            socket.write_all(&chunk).await.unwrap();
            socket.write_all(b"\r\n").await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        socket.write_all(b"0\r\n\r\n").await.unwrap();
        String::from_utf8_lossy(&request[..read]).into_owned()
    });
    (format!("http://{address}"), handle)
}

pub(crate) fn json_response(status: &str, headers: &[(&str, &str)], body: &str) -> String {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    response.push_str(body);
    response
}
