#[macro_use]
extern crate rocket;
use rocket::Request;
use rocket::data::Limits;
use rocket::fs::FileServer;
use rocket::http::{ContentType, Header};
use rocket::response::{Redirect, content::RawHtml};
use rocket::{
    Response,
    fairing::{Fairing, Info, Kind},
};
use rocket_dyn_templates::{Template, context};
use std::path::Path;
use std::path::PathBuf;
use storyteller::ApiClient;
use storyteller::Config;
use structopt::StructOpt;
use tokio::{fs::File, io::AsyncReadExt};
use ubyte::ByteUnit;
#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    #[structopt(short, long)]
    generate_config: bool,
    #[structopt(short, long)]
    config_file: Option<PathBuf>,
}

use dashmap::DashMap;
use uuid::Uuid;

type RequestCache = DashMap<Uuid, String>;

// there's no reason to store state in query param bc the only state the html page has is the template render which would be insecure.
#[catch(401)]
fn unauthorized(req: &rocket::Request<'_>) -> Redirect {
    let attempted_path = req.uri().to_string();
    let redirect = format!("/profiles/login?redirect={}", attempted_path);
    Redirect::to(redirect)
}

#[catch(404)]
fn notfound(req: &Request) -> RawHtml<Template> {
    let requested_uri = req.uri().to_string();
    RawHtml(Template::render(
        "notfound",
        context! { title: "404 page not found", page: requested_uri },
    ))
}

#[get("/")]
async fn index() -> RawHtml<Template> {
    RawHtml(Template::render(
        "index",
        context! { title: "Queer Respite" },
    ))
}

async fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, std::io::Error> {
    let mut file = File::open(path.as_ref()).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(serde_json::from_str(&contents).unwrap())
}

// Fairing to add custom mime type
pub struct WasmContentTypeFairing;

#[rocket::async_trait]
impl Fairing for WasmContentTypeFairing {
    fn info(&self) -> Info {
        Info {
            name: "Add .wasm Content-Type",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        if let Some(path) = req.uri().path().segments().last() {
            if path.ends_with(".wasm") {
                res.set_header(ContentType::new("application", "wasm"));
            }
        }
    }
}

#[launch]
async fn rocket() -> _ {
    let args = Args::from_args();

    let config = if let Some(config_file) = args.config_file {
        load_config(&config_file).await.unwrap()
    } else if args.generate_config {
        let config = Config::default();
        println!("{}", serde_json::to_string(&config).unwrap());
        std::process::exit(-1);
    } else {
        panic!("unable to fetch config");
    };

    let processor = storyteller::assets::images::ImageProcessor::new(
        config.url().to_string(),
        config.images.clone(),
    )
    .await;
    let rocketconfig = rocket::Config {
        address: config.listen().parse().unwrap(),
        port: config.port(),
        limits: Limits::new()
            .limit("file", ByteUnit::Gigabyte(2))
            .limit("form", ByteUnit::Gigabyte(2))
            .limit("data-form", ByteUnit::Gigabyte(2)),
        ..rocket::Config::default()
    };

    let url = config.api_endpoint();
    println!("api endpoint: {}", url);
    let api = ApiClient::new(&url).await.unwrap();
    let decoding_key = api.get_jwt_pubkey().await.unwrap();

    let mut validator = rocket_oidc::client::Validator::from_pubkey(
        config.api_endpoint().to_string(),
        "storyteller".to_string(),
        "RS256".to_string(),
        decoding_key,
    )
    .unwrap();
    for oidc in config.oidc().iter() {
        validator
            .extend_from_oidc(&oidc.issuer_url, "storyteller", "RS256")
            .await
            .unwrap();
    }

    let rocket = rocket::custom(rocketconfig)
        .manage(api)
        .manage(RequestCache::new())
        .attach(WasmContentTypeFairing)
        .manage(validator)
        .mount("/", routes![index])
        .manage(processor)
        .mount("/assets/graphs/", storyteller::assets::graphs::get_routes())
        .mount("/assets", storyteller::assets::get_routes())
        .mount("/stories", storyteller::stories::get_routes())
        .mount("/characters", storyteller::characters::get_routes())
        .mount("/timelines", storyteller::timelines::get_routes())
        .mount("/locations", storyteller::locations::get_routes())
        .mount("/relations", storyteller::relations::get_routes())
        .mount("/fragments", storyteller::fragments::get_routes())
        .mount("/organizations", storyteller::organizations::get_routes())
        .register("/", catchers![unauthorized, notfound])
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static"))
        .mount("/profiles", storyteller::profiles::get_routes())
        .mount("/notes/", storyteller::notes::get_routes())
        .mount("/assets/images/", storyteller::assets::images::get_routes())
        .mount("/search", storyteller::search::get_routes());

    //rocket_oidc::register_validator(rocket, validator);
    rocket_oidc::setup(rocket, config.oidc().iter().next().unwrap().clone())
        .await
        .unwrap()
}
