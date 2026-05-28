#[tokio::test]
async fn test_12_extractors_compilation() {
    use oxidite::prelude::*;

    async fn h12(
        _e1: State<()>, _e2: State<()>, _e3: State<()>, _e4: State<()>,
        _e5: State<()>, _e6: State<()>, _e7: State<()>, _e8: State<()>,
        _e9: State<()>, _e10: State<()>, _e11: State<()>, _e12: State<()>,
    ) -> Result<Response> {
        Ok(Response::text("ok"))
    }

    let mut router = Router::new();
    router.with_state(());
    router.get("/", h12);
}

#[test]
fn test_orm_error_not_found_id_type() {
    let err = oxidite::db::OrmError::NotFound {
        model: "User",
        id: "abc-123".to_string(),
    };
    assert_eq!(err.to_string(), "model `User` with id `abc-123` was not found");
}
