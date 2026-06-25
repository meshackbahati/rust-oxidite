use oxidite::prelude::*;
use oxidite::template::{Context, TemplateContext};
use std::sync::Arc;

pub fn register(router: &mut Router) {
    // Share template config via type-safe State extractor.
    // Handlers create the engine per-request — no global lifecycle coupling.
    let templates = Arc::new(TemplateContext::new("templates"));
    router.with_state(templates);

    router.get("/", index);
    router.get("/error-500", error_500);
    router.get("/error-400", error_400);
}

async fn index(_req: Request, templates: State<Arc<TemplateContext>>) -> Result<Response> {
    let mut context = Context::new();
    context.set("name", "Oxidite");

    let body = templates
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
