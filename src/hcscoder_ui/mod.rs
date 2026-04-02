//! hcscoder Terminal UI Module
//!
//! Interactive chat interface using ratatui.
//! Zero telemetry, no phone-home logic.

use crate::hcscoder_engine::query_engine::{add_to_conversation, create_conversation};
use crate::hcscoder_openrouter::client::{ChatMessage, HcscoderApiClient, MessageRole};
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};

/// Run the interactive chat interface
pub async fn run_chat_interface(
    api_key: Option<String>,
    model: String,
    initial_prompt: Option<String>,
    plain: bool,
) -> Result<()> {
    if plain || plain_env_preferred() {
        run_plain_chat(api_key, model, initial_prompt).await
    } else {
        run_tui_chat(api_key, model, initial_prompt).await
    }
}

/// Prefer line-oriented I/O when color must be disabled or the terminal cannot host a TUI well.
fn plain_env_preferred() -> bool {
    std::env::var_os("NO_COLOR").is_some()
        || std::env::var("TERM")
            .map(|t| t.eq_ignore_ascii_case("dumb"))
            .unwrap_or(false)
}

/// Plain text chat mode (no TUI)
async fn run_plain_chat(
    api_key: Option<String>,
    model: String,
    initial_prompt: Option<String>,
) -> Result<()> {
    use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

    let client = if let Some(key) = api_key {
        HcscoderApiClient::with_config(model, key, None)?
    } else {
        HcscoderApiClient::new(model)?
    };

    let mut messages = create_conversation("You are hcscoder, a helpful AI coding assistant.");

    // Remove the default system message and add proper one
    messages.clear();
    messages.push(ChatMessage {
        role: MessageRole::System,
        content: crate::hcscoder_engine::query_engine::HCS_CODER_SYSTEM_PROMPT.to_string(),
    });

    println!(
        "🚀 hcscoder v{} - Interactive Chat",
        env!("CARGO_PKG_VERSION")
    );
    println!("Model: {}", client.model());
    println!("Type 'quit' or 'exit' to end session");
    println!("{}", "=".repeat(50));
    println!();

    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut input = String::new();

    // Handle initial prompt if provided
    if let Some(prompt) = initial_prompt {
        println!("👤 You: {}", prompt);
        process_and_respond(&client, &mut messages, &prompt, &mut stdout).await?;
        stdout.flush().await?;
    }

    loop {
        input.clear();
        stdout.write_all("You: ".as_bytes()).await?;
        stdout.flush().await?;

        if stdin.read_line(&mut input).await? == 0 {
            break; // EOF
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("\n👋 Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("clear") {
            messages.truncate(1); // Keep only system message
            println!("✅ Conversation cleared.\n");
            continue;
        }

        if input.eq_ignore_ascii_case("help") {
            println!("\nCommands:");
            println!("  quit/exit - End session");
            println!("  clear     - Clear conversation history");
            println!("  help      - Show this help");
            println!();
            continue;
        }

        process_and_respond(&client, &mut messages, input, &mut stdout).await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn process_and_respond(
    client: &HcscoderApiClient,
    messages: &mut Vec<ChatMessage>,
    user_input: &str,
    stdout: &mut tokio::io::Stdout,
) -> Result<()> {
    use futures_util::stream::StreamExt;
    use tokio::io::AsyncWriteExt;

    // Add user message
    add_to_conversation(messages, MessageRole::User, user_input.to_string());

    // Stream response
    stdout.write_all("\nAssistant: ".as_bytes()).await?;
    stdout.flush().await?;

    let stream = crate::hcscoder_engine::query_engine::stream_query(
        client,
        messages.clone(),
        Some(0.7),
        Some(2000),
    )
    .await?;

    let mut response = String::new();
    tokio::pin!(stream);

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                stdout.write_all(text.as_bytes()).await?;
                stdout.flush().await?;
                response.push_str(&text);
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                break;
            }
        }
    }

    // Add assistant response to history
    add_to_conversation(messages, MessageRole::Assistant, response);

    stdout.write_all("\n\n".as_bytes()).await?;

    Ok(())
}

/// TUI-based chat interface (ratatui)
async fn run_tui_chat(
    api_key: Option<String>,
    model: String,
    initial_prompt: Option<String>,
) -> Result<()> {
    // Check if we're in a terminal
    if !atty::is(atty::Stream::Stdout) {
        return run_plain_chat(api_key, model, initial_prompt).await;
    }

    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use std::io;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = tui_main_loop(&mut terminal, api_key, model, initial_prompt).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn tui_main_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    api_key: Option<String>,
    model: String,
    initial_prompt: Option<String>,
) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode};

    let client = if let Some(key) = api_key {
        HcscoderApiClient::with_config(model, key, None)?
    } else {
        HcscoderApiClient::new(model)?
    };

    let mut messages: Vec<ChatMessage> = vec![ChatMessage {
        role: MessageRole::System,
        content: crate::hcscoder_engine::query_engine::HCS_CODER_SYSTEM_PROMPT.to_string(),
    }];
    let mut input_buffer = String::new();
    let mut response_buffer = String::new();
    let mut is_streaming = false;

    // Handle initial prompt
    if let Some(prompt) = initial_prompt {
        input_buffer = prompt;
    }

    loop {
        terminal.draw(|f| {
            ui(f, &messages, &input_buffer, &response_buffer, is_streaming);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('c')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        if !input_buffer.is_empty() && !is_streaming {
                            // Send message
                            is_streaming = true;
                            let user_input = input_buffer.clone();
                            input_buffer.clear();

                            messages.push(ChatMessage {
                                role: MessageRole::User,
                                content: user_input.clone(),
                            });

                            // Spawn streaming task
                            let _client_clone = client.clone();
                            let _messages_clone = messages.clone();

                            tokio::spawn(async move {
                                let _ = (_client_clone, _messages_clone);
                                // Handle streaming in background
                            });

                            response_buffer = "Thinking...".to_string();
                        }
                    }
                    KeyCode::Backspace => {
                        if !is_streaming {
                            input_buffer.pop();
                        }
                    }
                    KeyCode::Esc => {
                        if is_streaming {
                            is_streaming = false;
                            response_buffer.clear();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char(c) => {
                        if !is_streaming {
                            input_buffer.push(c);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(
    f: &mut Frame,
    messages: &[ChatMessage],
    input_buffer: &str,
    response_buffer: &str,
    is_streaming: bool,
) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Style},
        text::Line,
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),    // Messages
            Constraint::Length(3), // Input
        ])
        .split(f.size());

    // Messages area
    let mut message_lines = Vec::new();
    for msg in messages.iter().skip(1) {
        // Skip system message
        let prefix = match msg.role {
            MessageRole::User => "👤 ",
            MessageRole::Assistant => "🤖 ",
            MessageRole::System => "",
        };
        message_lines.push(Line::from(format!("{}{}", prefix, msg.content)));
    }

    if !response_buffer.is_empty() {
        message_lines.push(Line::from(format!("🤖 {}", response_buffer)));
    }

    let messages_widget = Paragraph::new(message_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("hcscoder Chat"),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(messages_widget, chunks[0]);

    // Input area
    let input_style = if is_streaming {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input_widget = Paragraph::new(input_buffer)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(if is_streaming {
                    "Streaming..."
                } else {
                    "Input (Enter to send, Ctrl+C to quit)"
                }),
        )
        .style(input_style);

    f.render_widget(input_widget, chunks[1]);
}

// Placeholder for atty check
mod atty {
    use std::io::IsTerminal;

    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        std::io::stdout().is_terminal()
    }
}
