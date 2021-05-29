use tide::{Server, Response};

mod store;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let app = create_app();
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn create_app() -> Server<store::Store> {
    let store = store::Store::new();
    let mut app = tide::Server::with_state(store);
    app.at("/:name").get(pop).put(push);
    app

}

async fn push(mut req: tide::Request<store::Store>) -> tide::Result<Response> {
    let msg = req.body_bytes().await?;
    req.state().push(req.param("name")?, msg).await;
    Ok(Response::new(200))
}

async fn pop(req: tide::Request<store::Store>) -> tide::Result<Response> {
    let timeout = match req.url().query_pairs()
        .filter(|(key, _)| key=="timeout")
        .map(|(_, value)| value)
        .next() {
        Some(timeout) => {
            Some(parse_duration::parse(timeout.as_ref())?)
        }
        None => None
    };
    match req.state()
        .pop(req.param("name")?, timeout)
        .await
    {
        None => Ok(Response::new(404)),
        Some(message) => Ok(Response::builder(200).body(message).build()),
    }
}

#[cfg(test)]
mod tests{
    use tide_testing::TideTestingExt;
    use super::*;

    #[async_std::test]
    async fn test_fifo(){
        let app = create_app();
        assert_eq!(app.get("/test").await.unwrap().status(), tide::StatusCode::NotFound);
        assert_eq!(app.put("/test").body("one".as_bytes()).await.unwrap().status(), tide::StatusCode::Ok);
        assert_eq!(app.put("/test").body("two".as_bytes()).await.unwrap().status(), tide::StatusCode::Ok);
        assert_eq!(app.get("/test").recv_string().await.unwrap(), "one");
        assert_eq!(app.get("/test").recv_string().await.unwrap(), "two");
        assert_eq!(app.get("/test").await.unwrap().status(), tide::StatusCode::NotFound);
    }

    #[async_std::test]
    async fn test_blocking(){
        let app = create_app();
        let app_clone = app.clone();
        assert_eq!(app.get("/test").await.unwrap().status(), tide::StatusCode::NotFound);
        let delayed_post = async_std::task::spawn(async move {
            async_std::task::sleep(std::time::Duration::from_secs(3)).await;
            assert_eq!(
                app_clone
                    .put("/test")
                    .body("one".as_bytes())
                    .await.unwrap()
                    .status(),
                tide::StatusCode::Ok,
            );
        });
        assert_eq!(app.get("/test").await.unwrap().status(), tide::StatusCode::NotFound);
        assert_eq!(app.get("/test?timeout=5s").recv_string().await.unwrap(), "one");
        delayed_post.await
    }
}