#![cfg(feature = "rocket_extras")]

use serde_json::Value;
use utoipa::OpenApi;

mod common;

#[test]
fn test_resolve_route_with_simple_url() {
    mod rocket_route_operation {
        use rocket::route;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[route(GET, uri = "/hello")]
        #[allow(unused)]
        fn hello() -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(handlers(rocket_route_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let operation = common::get_json_path(value, "paths./hello.get");

    assert_ne!(operation, &Value::Null, "expected paths.hello.get not null");
}

#[test]
fn test_resolve_get() {
    mod rocket_route_operation {
        use rocket::get;

        #[utoipa::path(responses(
                (status = 200, description = "Hello from server")
            ))]
        #[get("/hello/<id>/<name>?<colors>")]
        #[allow(unused)]
        fn hello(id: i32, name: &str, colors: Vec<&str>) -> String {
            "Hello".to_string()
        }
    }

    #[derive(OpenApi)]
    #[openapi(handlers(rocket_route_operation::hello))]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();
    let value = &serde_json::to_value(&openapi).unwrap();
    let parameters = common::get_json_path(value, r#"paths./hello/{id}/{digest}.get.parameters"#);

    assert_ne!(
        parameters,
        &Value::Null,
        "expected paths.hello.{{id}}.get.parameters not null"
    );

    dbg!(&openapi);
}
