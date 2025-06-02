#[macro_use]
extern crate rocket;
use rocket::Request;
use rocket::fs::FileServer;
use rocket::response::{Redirect, content::RawHtml};
use rocket_dyn_templates::{Template, context};
use std::path::Path;
use std::path::PathBuf;
use storyteller::ApiClient;
use storyteller::Config;
use structopt::StructOpt;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Clone, StructOpt)]
pub struct Args {
    #[structopt(short, long)]
    generate_config: bool,
    #[structopt(short, long)]
    config_file: Option<PathBuf>,
}

#[catch(401)]
fn unauthorized() -> Redirect {
    Redirect::to("/")
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

    let rocketconfig = rocket::Config {
        address: config.listen().parse().unwrap(),
        port: config.port(),
        ..rocket::Config::default()
    };

    let url = config.api_endpoint();
    println!("api endpoint: {}", url);
    let api = ApiClient::new(&url).await.unwrap();
    let rocket = rocket::custom(rocketconfig)
        .manage(api)
        .mount("/", routes![index])
        .mount("/stories", storyteller::stories::get_routes())
        .mount("/characters", storyteller::characters::get_routes())
        .mount("/places", storyteller::places::get_routes())
        .mount("/fragments", storyteller::fragments::get_routes())
        .register("/", catchers![unauthorized, notfound])
        .attach(Template::fairing())
        .mount("/static", FileServer::from("static"))
        .mount("/users", storyteller::users::get_routes())
        .mount("/search", storyteller::search::get_routes());
    rocket_oidc::setup(rocket, config.oidc().clone())
        .await
        .unwrap()
}
