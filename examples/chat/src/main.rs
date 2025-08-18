use anyhow::Result;
use async_openai::types::CreateChatCompletionRequestArgs;
use async_openai::{Client, config::OpenAIConfig};
use flows::runtime::tokio::TokioRuntime;
use futures::StreamExt;
use std::io::{self, Write};
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, BufReader, Stdin};

const CHANNEL_SIZE: usize = 8;
const DATA_CHANNEL_SIZE: usize = 16;

async fn example(
    init: (Client<OpenAIConfig>, String),
    ctrl: flows::FnController<TokioRuntime, String, CHANNEL_SIZE>,
    data: flows::FnDataHandle<(), String, DATA_CHANNEL_SIZE>,
) -> Result<()> {
    let messages = vec![
        async_openai::types::ChatCompletionRequestSystemMessageArgs::default()
            .content("You are a helpful assistant chatbot who responds to user conversations.")
            .build()
            .unwrap()
            .into(),
        async_openai::types::ChatCompletionRequestUserMessageArgs::default()
            .content(init.1)
            .build()
            .unwrap()
            .into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model("llama-3.1-8b-instant")
        .messages(messages)
        .build()
        .unwrap();

    let mut stream = init.0.chat().create_stream(request).await?;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                chunk.choices.iter().for_each(|chat_choice| {
                    let content = chat_choice.delta.content.clone();
                    match content {
                        Some(text) => {
                            print!("{}", text);
                        }
                        _ => {}
                    }
                });
            }
            Err(e) => {
                eprintln!("Error receiving chunk: {}", e);
                break;
            }
        }
    }
    Ok(())
}

static SLOT_1: std::sync::LazyLock<
    flows::Slot<String, (), String, CHANNEL_SIZE, DATA_CHANNEL_SIZE>,
> = std::sync::LazyLock::new(|| flows::Slot::default());

static RUNTIME: std::sync::LazyLock<TokioRuntime> =
    std::sync::LazyLock::new(|| TokioRuntime::new());

async fn prompt_user(stdin: Stdin) -> Option<String> {
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();

    print!("> ");
    io::stdout().flush().unwrap();

    match reader.read_line(&mut input).await {
        Ok(0) => None, // EOF
        Ok(_) => {
            let message = input.trim().to_string();
            if message.is_empty() {
                None
            } else {
                Some(message)
            }
        }
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            None
        }
    }
}

#[tokio::main]
async fn main() {
    let slot = &*SLOT_1;
    let runtime = &*RUNTIME;

    let (fn_data_handle, user_data_handle) = slot.handles();
    let (fn_ctrl, flow_func_ctrl, user_ctrl) = slot.ctrls(runtime);

    let api_key = std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY environment variable not set");

    let config = OpenAIConfig::default()
        .with_api_base("https://api.groq.com/openai/v1")
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let stdin = tokio::io::stdin();

    let init_message = prompt_user(stdin).await.unwrap();

    let future = example((client, init_message), fn_ctrl, fn_data_handle);
    let flow = flows::Flow::new(future, flow_func_ctrl);

    let handle = tokio::spawn(flow);
    handle.await;
    // loop {
    //     while let Some(incoming) = user_data_handle.recv() {
    //         print!("{}", incoming);
    //     }
    // }
}
