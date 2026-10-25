use actix_web::{App, HttpResponse, HttpServer, web};

#[actix_web::main]
async fn main() {
    let server = HttpServer::new(|| App::new().route("/", web::get().to(get_index)));
    println!("Serving on http://localhost:3000...");
    server
        .bind("0.0.0.0:3000")
        .expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}

async fn get_index() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        r#"
            <title>GCD Calculator</title>
            <form action="/gcd" method="post">
                <input type="text" name="n"/>
                <input type="text" name="m"/>
                <button type="submit">Compute GCD</button>
            </form>
            "#,
    )
}
