use oxidite::prelude::*;
use oxidite::template::{Context, TemplateEngine};

pub fn register(router: &mut Router) {
    router.get("/", index);
    router.get("/error-500", error_500);
    router.get("/error-400", error_400);
}

async fn index(_req: Request) -> Result<Response> {
    let mut engine = TemplateEngine::new();
    engine
        .load_dir("templates")
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    let mut context = Context::new();
    context.set("name", "Oxidite");

    let body = engine
        .render("index.html", &context)
        .map_err(|e| Error::InternalServerError(e.to_string()))?;

    Ok(Response::html(body))
}

async fn error_500(_req: Request) -> Result<Response> {
    Err(Error::InternalServerError(
        "intentional test 500 from example-project".to_string(),
    ))
}

async fn error_400(_req: Request) -> Result<Response> {
    Err(Error::BadRequest(
        "intentional test 400 from example-project".to_string(),
    ))
}
