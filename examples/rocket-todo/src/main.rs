use rocket::{get, routes, Build, Rocket};

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    rocket::build().mount("", routes![hello])
}

#[get("/hello")]
fn hello() -> String {
    "hello".to_string()
}
