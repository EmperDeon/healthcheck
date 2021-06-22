extern crate clap;

use clap::{Arg, App as Cli, ArgMatches};

trait Check {
  fn args<'a>(cli: Cli<'a, 'a>) -> Cli<'a, 'a>;
  fn check(args: &ArgMatches) -> Result<(), String>;
}

fn main() {
  dotenv::dotenv().ok();

  let cli = Cli::new("Healthchecks helper utility")
    .version("0.1.0")
    .author("EmperDeon <emperdeon@protonmail.com>")
    .about("Helps check health of apps and services");

  let cli = TimestampCheck::args(cli);
  let cli = AmqpCheck::args(cli);
  let cli = PostgresCheck::args(cli);
  let cli = RedisCheck::args(cli);
  let cli = HttpCheck::args(cli);
  let matches = cli.get_matches();

  std::process::exit(match run_checks(matches) {
    Ok(_) => 0,
    Err(err) => {
      eprintln!("Error: {:?}", err);
      1
    }
  });
}

fn run_checks(args: ArgMatches) -> Result<(), String> {
  if let Err(e) = TimestampCheck::check(&args) { return Err(e); }
  if let Err(e) = AmqpCheck::check(&args) { return Err(e); }
  if let Err(e) = PostgresCheck::check(&args) { return Err(e); }
  if let Err(e) = RedisCheck::check(&args) { return Err(e); }
  if let Err(e) = HttpCheck::check(&args) { return Err(e); }

  Ok(())
}

////
//  Common functions
////

fn parse_int_safe(str: String) -> i64 {
  str.chars()
    .filter_map(|a| a.to_digit(10))
    .filter_map(|a| char::from_digit(a, 10) )
    .collect::<String>()
    .parse().unwrap()
}

////
// Timestamp
////

struct TimestampCheck {}
impl Check for TimestampCheck {
  fn args<'a>(cli: Cli<'a, 'a>) -> Cli<'a, 'a> {
    cli
      .arg(
        Arg::with_name("timestamp")
          .long("timestamp")
          .help("Check if file specified with `timestamp-file` has timestamp older than `timestamp-timeout` seconds ago")
          .long_help("Check if specified file has timestamp older than `timestamp-timeout` seconds ago\
        File should contain one timestamp. Spaces and newlines are trimmed")
      )
      .arg(
        Arg::with_name("timestamp-timeout")
          .requires("timestamp")
          .long("timestamp-timeout")
          .help("Sets timeout for timestamp. Default: `20`. See `timestamp`")
          .takes_value(true)
      )
      .arg(
        Arg::with_name("timestamp-file")
          .requires("timestamp")
          .long("timestamp-file")
          .help("Sets file with timestamp. Default: `/app/tmp/health.all`. See `timestamp`")
          .takes_value(true)
      )
  }

  fn check(args: &ArgMatches) -> Result<(), String> {
    if !args.is_present("timestamp") { return Ok(()); }

    let file = args.value_of("timestamp-file").unwrap_or("/app/tmp/health.all");
    let timeout: i64 = args.value_of("timestamp-timeout").unwrap_or("20").parse().unwrap_or(20);
    let timestamp = parse_int_safe(std::fs::read_to_string(file).unwrap());

    let diff = chrono::offset::Utc::now().timestamp() - timestamp;

    if diff > timeout {
      Err(format!("Timestamp: Diff larger then timeout by {}", diff - timeout))
    } else {
      Ok(())
    }
  }
}

////
// AmqpQL
////

struct AmqpCheck {}
impl Check for AmqpCheck {
  fn args<'a>(cli: Cli<'a, 'a>) -> Cli<'a, 'a> {
    cli
      .arg(
        Arg::with_name("amqp")
          .long("amqp")
          .help("Connect to server. URL can be specified with `amqp-url` option or AMQP_URL env variable")
      )
      .arg(
        Arg::with_name("amqp-url")
          .requires("amqp")
          .long("amqp-url")
          .help("Sets url for connection. Default: `amqp://guest:guest@amqp:5432/amqp`. See `amqp`")
          .takes_value(true)
      )
  }

  fn check(args: &ArgMatches) -> Result<(), String> {
    if !args.is_present("amqp") { return Ok(()); }

    let url = dotenv::var("AMQP_URL").unwrap_or("amqp://guest:guest@amqp:5432/amqp".to_owned());
    let url = args.value_of("amqp-url").unwrap_or(url.as_str());

    let connection = amiquip::Connection::insecure_open(url);
    if let Err(e) = connection { return Err(format!("AMQP: {:?}", e)); }

    let channel = connection.unwrap().open_channel(None);

    match channel {
      Ok(_) => { Ok(()) }
      Err(e) => { Err(format!("AMQP: {:?}", e)) }
    }
  }
}

////
// PostgreSQL
////

struct PostgresCheck {}
impl Check for PostgresCheck {
  fn args<'a>(cli: Cli<'a, 'a>) -> Cli<'a, 'a> {
    cli
      .arg(
        Arg::with_name("postgres")
          .long("postgres")
          .help("Connect to server and run SELECT 1. URL can be specified with `postgres-url` option or POSTGRES_URL env variable")
      )
      .arg(
        Arg::with_name("postgres-url")
          .requires("postgres")
          .long("postgres-url")
          .help("Sets url for connection. Default: `postgres://postgres:postgres@postgres:5432/postgres`. See `postgres`")
          .takes_value(true)
      )
  }

  fn check(args: &ArgMatches) -> Result<(), String> {
    if !args.is_present("postgres") { return Ok(()); }

    let url = dotenv::var("POSTGRES_URL").unwrap_or("postgres://postgres:postgres@postgres:5432/postgres".to_owned());
    let url = args.value_of("postgres-url").unwrap_or(url.as_str());

    let client = postgres::Client::connect(url, postgres::NoTls);
    if let Err(e) = client { return Err(format!("Postgres: {}", e)); }

    let result = client.unwrap().query("SELECT 1", &[]);

    match result {
      Ok(_) => { Ok(()) }
      Err(e) => { Err(format!("Postgres: {}", e)) }
    }
  }
}

////
// Redis
////

struct RedisCheck {}
impl Check for RedisCheck {
  fn args<'a>(cli: Cli<'a, 'a>) -> Cli<'a, 'a> {
    cli
      .arg(
        Arg::with_name("redis")
          .long("redis")
          .help("Connect to server and run INFO server. URL can be specified with `redis-url` option or REDIS_URL env variable")
      )
      .arg(
        Arg::with_name("redis-url")
          .requires("redis")
          .long("redis-url")
          .help("Sets url of server. Default: `redis://redis:6379/0`. See `redis`")
          .takes_value(true)
      )
  }

  fn check(args: &ArgMatches) -> Result<(), String> {
    if !args.is_present("redis") { return Ok(()); }

    let url = dotenv::var("REDIS_URL").unwrap_or("redis://redis:6379/0".to_owned());
    let url = args.value_of("redis-url").unwrap_or(url.as_str());

    let client = redis::Client::open(url).unwrap();
    let con = client.get_connection();
    if let Err(e) = con { return Err(format!("Redis: {}", e)); }

    let result: redis::RedisResult<String> = redis::cmd("INFO").arg("server").query(&mut con.unwrap());
    match result {
      Ok(_) => { Ok(()) }
      Err(e) => { Err(format!("Redis: {}", e)) }
    }
  }
}

////
// Http
////

struct HttpCheck {}
impl Check for HttpCheck {
  fn args<'a>(cli: Cli<'a, 'a>) -> Cli<'a, 'a> {
    cli
      .arg(
        Arg::with_name("http")
          .long("http")
          .help("Request and await a 200 response. URL can be specified with `http-url` option or HTTP_URL env variable")
      )
      .arg(
        Arg::with_name("http-url")
          .requires("http")
          .long("http-url")
          .help("Sets url of server. Default: `http://localhost:8080`. See `http`")
          .takes_value(true)
      )
  }

  fn check(args: &ArgMatches) -> Result<(), String> {
    if !args.is_present("http") { return Ok(()); }

    let url = args.value_of("http-url").unwrap_or("http://localhost:8080");

    match ureq::get(url).call() {
      Ok(_) => { Ok(()) }
      Err(e) => { Err(format!("Http: {}", e)) }
    }
  }
}
