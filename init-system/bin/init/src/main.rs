use tokio::net::UnixListener;

mod group;
mod service;
mod service_worker;
mod settings;


#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?; // Install panic handler

    let settings = settings::Settings::load()?;
    println!("{:#?}", settings);

    let socket = UnixListener::bind(&settings.listen)?;

    loop {
        let (stream, _) = socket.accept().await?;
        tokio::spawn(async move {

        });
    }
}
