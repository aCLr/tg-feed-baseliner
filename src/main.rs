use anyhow::Result;
use rust_tdlib::{client::{Client, Worker}, tdjson, types::TdlibParameters};
use rust_tdlib::client::ClientState;
use rust_tdlib::client::tdlib_client::TdJson;
use rust_tdlib::types::{ChatType, GetChat, GetChats, GetMessages};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    tdjson::set_log_verbosity_level(std::env::var("TDLIB_VERBOSITY").unwrap_or("1".to_string()).parse()?);

    let tdlib_parameters = TdlibParameters::builder()
        .database_directory("tddb")
        .use_test_dc(false)
        .api_id(std::env::var("API_ID")?.parse::<i32>().unwrap())
        .api_hash(std::env::var("API_HASH")?)
        .system_language_code("en")
        .device_model("Desktop")
        .system_version("Unknown")
        .application_version(env!("CARGO_PKG_VERSION"))
        .enable_storage_optimizer(true)
        .build();

    let client = Client::builder()
        .with_tdlib_parameters(tdlib_parameters)
        .build()?;


    let mut worker = Worker::builder().build()?;
    let mut waiter = worker.start();

    let client: Client<TdJson> = tokio::select! {
        c = worker.bind_client(client) => {
            match c {
                Ok(cl) => cl,
                Err(e) => panic!("{:?}", e)
            }
        }
        w = &mut waiter => panic!("{:?}", w),
    };
    loop {
        if worker.wait_client_state(&client).await? == ClientState::Opened {
            println!("client authorized");
            break;
        }
    }

    println!("wait for chats");
    let chats = client.get_chats(GetChats::builder().limit(10).build()).await?;
    println!("got chats");

    for &chat_id in chats.chat_ids() {
        println!("wait for a chat");
        let chat = client.get_chat(GetChat::builder().chat_id(chat_id).build()).await?;

        if let ChatType::Supergroup(sg) = chat.type_(){
            if !sg.is_channel() {
                println!("not a channel");
                continue
            }

            println!("wait for chat messages");
            let messages = client.get_messages(GetMessages::builder().chat_id(chat_id).build()).await?;
            println!("chat messages received");
            println!("{messages:?}");
        }

        break;
    }

    Ok(())
}
