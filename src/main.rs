use axum::Router;
use std::{
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tower_http::services::ServeDir;
use wry::WebViewBuilder;

fn install_webview_resources(path: &Path) {
    // TODO: implement the actual function
    let hello_world = r#"
<!DOCTYPE html>
<html>
  <body>
  	Hello World!
  </body>
</html>
    "#;
    std::fs::File::create(path.join("index.html"))
        .unwrap()
        .write_all(hello_world.as_bytes())
        .unwrap();
}

fn ensure_app_dir() -> PathBuf {
    let mut app_dir = dirs::data_dir().expect("Failed to get data dir");
    app_dir.push("wry_example");

    if !app_dir.exists() {
        app_dir.push("www");
        std::fs::create_dir_all(&app_dir).expect("Failed to create an app dir in data dir");
        install_webview_resources(&app_dir);
        app_dir.pop();
    }

    app_dir
}

async fn local_http_server_main(port_tx: tokio::sync::oneshot::Sender<u16>, app_dir: PathBuf) {
    let webview_resources = app_dir.join("www");
    let app = Router::new().nest_service("/", ServeDir::new(webview_resources));
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    port_tx.send(listener.local_addr().unwrap().port()).unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Look Kirill, no install!")
        .build(&event_loop)
        .unwrap();

    let (port_tx, port_rx) = tokio::sync::oneshot::channel::<u16>();

    let _local_http_server_handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        let app_dir = ensure_app_dir();
        rt.block_on(local_http_server_main(port_tx, app_dir));
    });

    let port: u16 = port_rx.blocking_recv().unwrap();

    // starting the webview
    let _webview = WebViewBuilder::new(&window)
        .with_url(&format!("http://localhost:{port}/"))?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
