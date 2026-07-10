use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn read_request(socket: &mut tokio::net::TcpStream) -> String {
    let mut request = Vec::new();
    loop {
        let mut chunk = [0; 4096];
        let read = socket.read(&mut chunk).await.unwrap();
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        if let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if request.len() >= header_end + 4 + content_length {
                break;
            }
        }
    }
    String::from_utf8_lossy(&request).into_owned()
}

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
        let request = read_request(&mut socket).await;
        socket.write_all(response.as_bytes()).await.unwrap();
        request
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

#[cfg(feature = "stream")]
pub(crate) async fn chunked_server(
    path_prefix: &str,
    headers: &[(&str, &str)],
    chunks: Vec<Vec<u8>>,
) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let mut response_headers = String::from(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n",
    );
    for (name, value) in headers {
        response_headers.push_str(name);
        response_headers.push_str(": ");
        response_headers.push_str(value);
        response_headers.push_str("\r\n");
    }
    response_headers.push_str("\r\n");
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let request = read_request(&mut socket).await;
        socket.write_all(response_headers.as_bytes()).await.unwrap();
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
        request
    });
    (
        format!("http://{address}/{}", path_prefix.trim_matches('/')),
        handle,
    )
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
