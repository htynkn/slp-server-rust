#[macro_use]
extern crate lazy_static;

mod graphql;
mod slp;
mod new_slp;
mod graphql_ws_filter;
mod util;
mod plugin;
#[cfg(test)]
mod test;
mod panic;

use graphql::{schema, Context};
use std::net::SocketAddr;
use serde::Serialize;
use std::convert::Infallible;
use graphql_ws_filter::make_graphql_ws_filter;
use warp::{Filter, filters::BoxedFilter, http::Method};
use env_logger::Env;
use clap::{Arg, App, ArgMatches};
use tower::{ServiceBuilder, Service, service_fn};
use new_slp::{Server, InPacket, SendTo, service};

#[derive(Serialize)]
struct Info {
    online: i32,
    version: String,
}

async fn server_info(context: Context) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&context.udp_server.server_info().await))
}

fn make_state(context: &Context) -> BoxedFilter<(Context,)> {
    let ctx = context.clone();
    warp::any().map(move || ctx.clone()).boxed()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("slp_server_rust=info")).init();
    panic::set_panic_hook();

    let maker = ServiceBuilder::new().service(service_fn(service));
    let mut server = Server::new(maker).await.unwrap();
    let bind_address = format!("{}:{}", "0.0.0.0", 12345);
    let socket_addr: SocketAddr = bind_address.parse().unwrap();
    tokio::spawn(async move {
        server.serve(socket_addr).await.unwrap();
    });

    let matches = get_matches();

    let port: u16 = matches.value_of("port").unwrap_or("11451").parse().expect("Can't parse port");
    let ignore_idle = matches.is_present("ignore_idle");
    let simple_auth = matches.value_of("simple_auth");
    if ignore_idle {
        log::info!("--ignore-idle is not tested, bugs are expected");
    }
    let admin_token = matches.value_of("admin_token").map(str::to_string);

    let bind_address = format!("{}:{}", "0.0.0.0", port);
    let socket_addr: &SocketAddr = &bind_address.parse().unwrap();

    // TODO add cors
    // let context = Context::new(udp_server, admin_token);

    // log::info!("Listening on {}", bind_address);

    // let graphql_filter = juniper_warp::make_graphql_filter(schema(), make_state(&context));
    // let graphql_ws_filter = make_graphql_ws_filter(schema(), make_state(&context));


    // let log = warp::log("warp_server");
    // let routes = (
    //     warp::path("info")
    //         .and(make_state(&context))
    //         .and_then(server_info)
    //     .or(warp::post()
    //         .and(graphql_filter))
    //     .or(
    //         warp::get()
    //         .and(graphql_ws_filter))
    // )
    // .or(warp::get()
    //     .and(juniper_warp::playground_filter("/", Some("/"))))
    //     .with(log);

    // warp::serve(routes)
    //     .run(*socket_addr)
    //     .await;

    Ok(())
}

fn get_matches<'a>() -> ArgMatches<'a> {
    let version = format!("{}-{}", std::env!("CARGO_PKG_VERSION"), std::env!("GIT_HASH"));
    App::new("slp-server-rust")
        .version(&*version)
        .author("imspace <spacemeowx2@gmail.com>")
        .about("switch-lan-play Server written in Rust")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("Port")
            .help("Sets server listening port")
            .takes_value(true))
        .arg(Arg::with_name("admin_token")
            .long("admin-token")
            .value_name("Admin Token")
            .help("Token for admin query. If not preset, no one can query admin information."))
        .arg(Arg::with_name("ignore_idle")
            .short("i")
            .long("ignore-idle")
            .help("Don't send broadcast to idle clients"))
        .arg(Arg::with_name("simple_auth")
            .long("simple-auth")
            .help("pass the auth via username and password, or else there's no authentication."))
        .get_matches()
}
