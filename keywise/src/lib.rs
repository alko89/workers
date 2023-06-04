use serde::{Deserialize, Serialize};
use worker::*;
use utils::cors::CorsHeaders;
use utils::error::respond_error;

static BASE_URL: &str = "https://kodadot.xyz/";

#[derive(Deserialize, Serialize)]
struct Key {
    key: String,
}

#[derive(Deserialize, Serialize)]
struct KeyValue {
    key: String,
    url: String,
}

fn root(_: Request, _: RouteContext<()>) -> Result<Response> {
    Response::ok("KodaDot URL Shortener")
}

async fn redirect_by_key<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    let key = ctx.param("key").unwrap();
    let list = ctx.kv("list")?;
    return match list.get(key).text().await? {
        Some(value) => Response::redirect(
            Url::parse(&format!("{}{}", BASE_URL.to_owned(), value))?
        ),
        None => respond_error("Key not found", 404),
    };
}

async fn resolve_key<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    let key = ctx.param("key").unwrap();
    let list = ctx.kv("list")?;
    return match list.get(key).text().await? {
        Some(value) => CorsHeaders::update(
            Response::from_json(&KeyValue { key: key.to_owned(), url: value })
        ),
        None => respond_error("Key not found", 404),
    };
}

async fn create_key<D>(mut req: Request, ctx: RouteContext<D>) -> Result<Response> {
    let body: KeyValue = req.json().await?;
    let list = ctx.kv("list")?;
    
    return match list.get(&body.key).text().await? {
        Some(_) => respond_error("Key already exists", 409),
        None => {
            list.put(&body.key, body.url)?.execute().await?;
            Response::from_json(&Key { key: body.key })
        }
    };
}

async fn delete_key<D>(_: Request, ctx: RouteContext<D>) -> Result<Response> {
    let key = ctx.param("key").unwrap();
    let list = ctx.kv("list")?;
    list.delete(&key).await?;
    return Response::from_json(&Key { key: key.to_string() });
}

fn empty_response<D>(_: Request, _: RouteContext<D>) ->  Result<Response> {
    CorsHeaders::response()
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    CorsHeaders::update(router
        .get("/", root)
        .get_async("/resolve/:key", resolve_key)
        .get_async("/:key", redirect_by_key)
        .post_async("/", create_key)
        .delete_async("/:key", delete_key)
        .options("/*pathname", empty_response)
        .run(req, env)
        .await)
}
