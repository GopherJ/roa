use async_std::fs::read_to_string;
use async_std::task::spawn;
use http::header::ACCEPT_ENCODING;
use roa::body::DispositionType;
use roa::compress::Compress;
use roa::preload::*;
use roa::router::Router;
use roa::App;

#[tokio::test]
async fn serve_static_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(());
    app.call(|mut ctx| async move {
        ctx.write_file("assets/author.txt", DispositionType::Inline)
            .await
    });
    let (addr, server) = app.run()?;
    spawn(server);
    let resp = reqwest::get(&format!("http://{}", addr)).await?;
    assert_eq!("Hexilee", resp.text().await?);
    Ok(())
}

#[tokio::test]
async fn serve_router_variable() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::<()>::new();
    router.get("/:filename", |mut ctx| async move {
        let filename = ctx.must_param("filename")?;
        ctx.write_file(format!("assets/{}", &*filename), DispositionType::Inline)
            .await
    });
    let mut app = App::new(());
    app.gate(router.routes("/")?);
    let (addr, server) = app.run()?;
    spawn(server);
    let resp = reqwest::get(&format!("http://{}/author.txt", addr)).await?;
    assert_eq!("Hexilee", resp.text().await?);
    Ok(())
}

#[tokio::test]
async fn serve_router_wildcard() -> Result<(), Box<dyn std::error::Error>> {
    let mut router = Router::<()>::new();
    router.get("/*{path}", |mut ctx| async move {
        let path = ctx.must_param("path")?;
        ctx.write_file(format!("./{}", &*path), DispositionType::Inline)
            .await
    });
    let mut app = App::new(());
    app.gate(router.routes("/")?);
    let (addr, server) = app.run()?;
    spawn(server);
    let resp = reqwest::get(&format!("http://{}/assets/author.txt", addr)).await?;
    assert_eq!("Hexilee", resp.text().await?);
    Ok(())
}

#[tokio::test]
async fn serve_gzip() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(());
    app.gate(Compress::default()).end(|mut ctx| async move {
        ctx.write_file("assets/welcome.html", DispositionType::Inline)
            .await
    });
    let (addr, server) = app.run()?;
    spawn(server);
    let client = reqwest::Client::builder().gzip(true).build()?;
    let resp = client
        .get(&format!("http://{}", addr))
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .await?;

    assert_eq!(
        read_to_string("assets/welcome.html").await?,
        resp.text().await?
    );
    Ok(())
}
