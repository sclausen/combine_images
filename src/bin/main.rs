#[macro_use]
extern crate rocket;

use combine_images::{create_path_from_current_dir, process_image};
use rocket::{
    fs::FileServer,
    serde::{json::Json, Deserialize, Serialize},
};

#[derive(Deserialize)]
struct ImageUrls {
    background: String,
    overlay: String,
}

#[derive(Serialize)]
struct OutputFilename {
    output: String,
}

#[post("/process", data = "<image_urls>")]
async fn process_images(
    image_urls: Json<ImageUrls>,
    config: &rocket::Config,
) -> Json<OutputFilename> {
    match process_image(image_urls.background.clone(), image_urls.overlay.clone()).await {
        Ok(filename) => Json(OutputFilename {
            output: format!(
                "http://{}:{}/images/{}",
                config.address, config.port, filename
            ),
        }),
        Err(_) => Json(OutputFilename {
            output: String::from("Error processing images"),
        }),
    }
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build()
        .mount(
            "/images",
            FileServer::from(create_path_from_current_dir("images").unwrap()),
        )
        .mount("/", routes![process_images]);
    rocket
}
