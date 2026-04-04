//! hcscoder Terminal UI Module
//!
//! Interactive chat interface using ratatui with theming support.
//! Zero telemetry, no phone-home logic.
//!
//! ## Themes
//! - Default: Blue/White professional look
//! - Dracula: Dark with pink/purple accents
//! - Gruvbox: Warm earth tones
//! - Nord: Cool blue tones
//! - HighContrast: Accessibility focused

use crate::hcscoder_engine::query_engine::{add_to_conversation, create_conversation};
use crate::hcscoder_openrouter::client::{ChatMessage, HcscoderApiClient, MessageRole};
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Available UI themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTheme {
    Default,
    Dracula,
    Gruvbox,
    Nord,
    HighContrast,
}

impl UiTheme {
    pub fn from_env() -> Self {
        std::env::var("HCSCODER_THEME")
            .ok()
            .map(|s| s.to_lowercase())
            .map_or(UiTheme::Default, |s| match s.as_str() {
                "dracula" => UiTheme::Dracula,
                "gruvbox" => UiTheme::Gruvbox,
                "nord" => UiTheme::Nord,
                "highcontrast" | "high_contrast" => UiTheme::HighContrast,
                _ => UiTheme::Default,
            })
    }

    pub fn primary_color(self) -> ratatui::style::Color {
        match self {
            UiTheme::Default => ratatui::style::Color::Blue,
            UiTheme::Dracula => ratatui::style::Color::Magenta,
            UiTheme::Gruvbox => ratatui::style::Color::Yellow,
            UiTheme::Nord => ratatui::style::Color::Cyan,
            UiTheme::HighContrast => ratatui::style::Color::White,
        }
    }

    pub fn secondary_color(self) -> ratatui::style::Color {
        match self {
            UiTheme::Default => ratatui::style::Color::White,
            UiTheme::Dracula => ratatui::style::Color::Cyan,
            UiTheme::Gruvbox => ratatui::style::Color::Green,
            UiTheme::Nord => ratatui::style::Color::Blue,
            UiTheme::HighContrast => ratatui::style::Color::Yellow,
        }
    }

    pub fn border_color(self) -> ratatui::style::Color {
        self.primary_color()
    }

    pub fn error_color(self) -> ratatui::style::Color {
        match self {
            UiTheme::HighContrast => ratatui::style::Color::Red,
            _ => ratatui::style::Color::Red,
        }
    }
}

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
    // Check for NO_COLOR environment variable
    if std::env::var_os("NO_COLOR").is_some() {
        return true;
    }
    
    // Check for dumb terminal
    if std::env::var("TERM")
        .map(|t| t.eq_ignore_ascii_case("dumb"))
        .unwrap_or(false)
    {
        return true;
    }
    
    // Check for Termux (Android) - often has limited TUI support
    if std::env::var("PREFIX")
        .map(|p| p.contains("com.termux"))
        .unwrap_or(false)
    {
        return true;
    }
    
    false
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

    // Print ASCII logo
    println!("{}", crate::LOGO_ASCII);
    println!("🚀 hcscoder v{} - Interactive Chat", env!("CARGO_PKG_VERSION"));
    println!("   Made with ❤️  by hcsmedia | Stable Release");
    println!("Model: {}", client.model());
    println!("Theme: {:?}", UiTheme::from_env());
    println!("Type 'quit', 'exit', 'clear', 'help', or 'theme' for commands");
    println!("{}", "─".repeat(60));
    println!();

    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut input = String::new();

    // Handle initial prompt if provided
    if let Some(prompt) = initial_prompt {
        println!("👤 You: {}\n", prompt);
        process_and_respond(&client, &mut messages, &prompt, &mut stdout).await?;
        stdout.flush().await?;
    }

    loop {
        input.clear();
        print!("👤 You: ");
        stdout.flush().await?;

        if stdin.read_line(&mut input).await? == 0 {
            break; // EOF
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("\n👋 Goodbye! Thanks for using hcscoder by hcsmedia ❤️ | Made with love");
            break;
        }

        if input.eq_ignore_ascii_case("clear") {
            messages.truncate(1); // Keep only system message
            println!("\n✅ Conversation cleared.\n");
            continue;
        }

        if input.eq_ignore_ascii_case("help") {
            println!("\n📖 Commands:");
            println!("  quit/exit  - End session");
            println!("  clear      - Clear conversation history");
            println!("  help       - Show this help");
            println!("  theme      - Show current theme");
            println!("  themes     - List available themes");
            println!("  model      - Show current model");
            println!("  status     - Show connection status");
            println!();
            continue;
        }

        if input.eq_ignore_ascii_case("theme") {
            println!("\n🎨 Current theme: {:?}", UiTheme::from_env());
            println!("   Set HCSCODER_THEME environment variable to change:");
            println!("   - default, dracula, gruvbox, nord, highcontrast");
            println!();
            continue;
        }

        if input.eq_ignore_ascii_case("themes") {
            println!("\n🎨 Available themes:");
            println!("  default       - Blue/White professional look");
            println!("  dracula       - Dark with pink/purple accents");
            println!("  gruvbox       - Warm earth tones");
            println!("  nord          - Cool blue tones");
            println!("  highcontrast  - Accessibility focused");
            println!();
            continue;
        }

        if input.eq_ignore_ascii_case("model") {
            println!("\n🤖 Current model: {}", client.model());
            println!();
            continue;
        }

        if input.eq_ignore_ascii_case("status") {
            println!("\n📊 Status: Connected to OpenRouter");
            println!("   Model: {}", client.model());
            println!("   Theme: {:?}", UiTheme::from_env());
            println!("   Messages in history: {}", messages.len());
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

    // Stream response with emoji indicator
    stdout.write_all("\n🤖 Assistant: ".as_bytes()).await?;
    stdout.flush().await?;

    let stream = crate::hcscoder_engine::query_engine::stream_query(
        client,
        messages.clone(),
        Some(0.7),
        Some(2000),
    )
    .await?;

    let mut response = String::new();
    let mut char_count = 0;
    tokio::pin!(stream);

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                stdout.write_all(text.as_bytes()).await?;
                stdout.flush().await?;
                response.push_str(&text);
                char_count += text.len();
            }
            Err(e) => {
                eprintln!("\n❌ Error: {}", e);
                break;
            }
        }
    }

    // Add assistant response to history
    add_to_conversation(messages, MessageRole::Assistant, response);

    // Show token usage info (rough estimate)
    let estimated_tokens = crate::hcscoder_engine::query_engine::estimate_tokens(&response);
    stdout
        .write_all(format!("\n\n   [~{} tokens]\n\n", estimated_tokens).as_bytes())
        .await?;

    Ok(())
}

/// TUI-based chat interface (ratatui)
async fn run_tui_chat(
    api_key: Option<String>,
    model: String,
    initial_prompt: Option<String>,
) -> Result<()> {
    // Check if we're in a terminal using std::io::IsTerminal (modern approach)
    use std::io::IsTerminal;
    if !std::io::stdout().is_terminal() {
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

/// Message types for TUI communication
#[derive(Debug, Clone)]
enum TuiMessage {
    UserInput(String),
    AssistantResponse(String),
    Error(String),
    StreamingChunk(String),
    StreamingComplete,
}

/// TUI State for managing the interface
struct TuiState {
    messages: Vec<ChatMessage>,
    input_buffer: String,
    response_buffer: String,
    is_streaming: bool,
    scroll_offset: usize,
    theme: UiTheme,
    status_message: Option<String>,
}

impl TuiState {
    fn new(system_prompt: String, theme: UiTheme) -> Self {
        Self {
            messages: vec![ChatMessage {
                role: MessageRole::System,
                content: system_prompt,
            }],
            input_buffer: String::new(),
            response_buffer: String::new(),
            is_streaming: false,
            scroll_offset: 0,
            theme,
            status_message: None,
        }
    }

    fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content,
        });
        // Reset scroll to bottom when new message arrives
        self.scroll_offset = 0;
    }

    fn start_streaming(&mut self) {
        self.is_streaming = true;
        self.response_buffer.clear();
        self.status_message = Some("Thinking...".to_string());
    }

    fn append_chunk(&mut self, chunk: String) {
        self.response_buffer.push_str(&chunk);
        // Auto-scroll to bottom
        self.scroll_offset = 0;
    }

    fn finish_streaming(&mut self) {
        if !self.response_buffer.is_empty() {
            self.add_to_history(MessageRole::Assistant, self.response_buffer.clone());
        }
        self.is_streaming = false;
        self.status_message = None;
    }

    fn add_to_history(&mut self, role: MessageRole, content: String) {
        self.messages.push(ChatMessage { role, content });
        self.scroll_offset = 0;
    }

    fn set_error(&mut self, error: String) {
        self.is_streaming = false;
        self.status_message = Some(format!("❌ {}", error));
    }

    fn visible_messages(&self, max_height: usize) -> Vec<&ChatMessage> {
        let skip = self.scroll_offset;
        let user_messages: Vec<_> = self.messages.iter().skip(1).collect(); // Skip system
        if skip >= user_messages.len() {
            user_messages
        } else {
            user_messages.into_iter().take(max_height.min(user_messages.len() - skip)).collect()
        }
    }
}

async fn tui_main_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    api_key: Option<String>,
    model: String,
    initial_prompt: Option<String>,
) -> Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use futures_util::stream::StreamExt;

    let client = if let Some(key) = api_key {
        HcscoderApiClient::with_config(model, key, None)?
    } else {
        HcscoderApiClient::new(model)?
    };

    let theme = UiTheme::from_env();
    let mut state = TuiState::new(
        crate::hcscoder_engine::query_engine::HCS_CODER_SYSTEM_PROMPT.to_string(),
        theme,
    );

    // Channel for streaming responses
    let (tx, mut rx) = mpsc::channel::<TuiMessage>(32);

    // Handle initial prompt
    if let Some(prompt) = initial_prompt {
        state.add_user_message(prompt.clone());
        let tx_clone = tx.clone();
        let client_clone = client.clone();
        let messages_clone = state.messages.clone();

        tokio::spawn(async move {
            stream_response(tx_clone, client_clone, messages_clone, Some(0.7), Some(2000)).await;
        });
        state.start_streaming();
    }

    loop {
        // Draw the UI
        terminal.draw(|f| ui(f, &state))?;

        // Poll for events with timeout
        let poll_duration = std::time::Duration::from_millis(50);
        
        tokio::select! {
            // Handle keyboard events
            _ = tokio::task::spawn_blocking(move || {
                if event::poll(poll_duration).unwrap_or(false) {
                    event::read().ok()
                } else {
                    None
                }
            }) => {
                if let Ok(Some(Event::Key(key))) = tokio::task::spawn_blocking(|| {
                    if event::poll(std::time::Duration::from_millis(10)).unwrap_or(false) {
                        event::read().ok()
                    } else {
                        None
                    }
                }).await {
                    // Only handle key press events (not release/repeat)
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('c')
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            if !state.input_buffer.is_empty() && !state.is_streaming {
                                // Send message
                                let user_input = state.input_buffer.clone();
                                state.input_buffer.clear();
                                state.add_user_message(user_input.clone());

                                // Spawn streaming task
                                let tx_clone = tx.clone();
                                let client_clone = client.clone();
                                let messages_clone = state.messages.clone();

                                tokio::spawn(async move {
                                    stream_response(tx_clone, client_clone, messages_clone, Some(0.7), Some(2000)).await;
                                });

                                state.start_streaming();
                            }
                        }
                        KeyCode::Backspace => {
                            if !state.is_streaming {
                                state.input_buffer.pop();
                            }
                        }
                        KeyCode::Esc => {
                            if state.is_streaming {
                                // Cannot cancel mid-stream easily, just clear buffer
                                state.response_buffer.clear();
                                state.finish_streaming();
                            } else {
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => {
                            if !state.is_streaming {
                                state.input_buffer.push(c);
                            }
                        }
                        KeyCode::Up => {
                            // Scroll up through messages
                            state.scroll_offset = state.scroll_offset.saturating_add(1);
                        }
                        KeyCode::Down => {
                            // Scroll down through messages
                            state.scroll_offset = state.scroll_offset.saturating_sub(1);
                        }
                        KeyCode::PageUp => {
                            state.scroll_offset = state.scroll_offset.saturating_add(10);
                        }
                        KeyCode::PageDown => {
                            state.scroll_offset = state.scroll_offset.saturating_sub(10);
                        }
                        _ => {}
                    }
                }
            }
            // Handle streaming messages
            Some(msg) = rx.recv() => {
                match msg {
                    TuiMessage::StreamingChunk(chunk) => {
                        state.append_chunk(chunk);
                    }
                    TuiMessage::AssistantResponse(full_response) => {
                        state.finish_streaming();
                    }
                    TuiMessage::Error(error) => {
                        state.set_error(error);
                    }
                    TuiMessage::StreamingComplete => {
                        state.finish_streaming();
                    }
                    TuiMessage::UserInput(_) => {
                        // Handled elsewhere
                    }
                }
            }
        }
    }
}

/// Stream response from API and send chunks via channel
async fn stream_response(
    tx: mpsc::Sender<TuiMessage>,
    client: HcscoderApiClient,
    messages: Vec<ChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
) {
    match crate::hcscoder_engine::query_engine::stream_query(&client, messages, temperature, max_tokens).await {
        Ok(stream) => {
            let mut stream = stream;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(text) => {
                        if tx.send(TuiMessage::StreamingChunk(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(TuiMessage::Error(e.to_string())).await;
                        break;
                    }
                }
            }
            let _ = tx.send(TuiMessage::StreamingComplete).await;
        }
        Err(e) => {
            let _ = tx.send(TuiMessage::Error(e.to_string())).await;
        }
    }
}

fn ui(f: &mut Frame, state: &TuiState) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Style, Modifier},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState},
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0),    // Messages
            Constraint::Length(3), // Input
        ])
        .split(f.size());

    // Messages area with theme colors
    let mut message_lines = Vec::new();
    for msg in state.messages.iter().skip(1) {
        // Skip system message
        let prefix = match msg.role {
            MessageRole::User => "👤 ",
            MessageRole::Assistant => "🤖 ",
            MessageRole::System => "",
        };
        
        let role_style = match msg.role {
            MessageRole::User => Style::default()
                .fg(state.theme.secondary_color())
                .add_modifier(Modifier::BOLD),
            MessageRole::Assistant => Style::default()
                .fg(state.theme.primary_color())
                .add_modifier(Modifier::BOLD),
            MessageRole::System => Style::default(),
        };
        
        message_lines.push(Line::from(vec![
            Span::styled(prefix, role_style),
            Span::raw(msg.content.clone()),
        ]));
    }

    // Add current streaming response
    if !state.response_buffer.is_empty() {
        message_lines.push(Line::from(vec![
            Span::styled("🤖 ", Style::default().fg(state.theme.primary_color())),
            Span::raw(state.response_buffer.clone()),
        ]));
    }

    // Add status message if present
    if let Some(status) = &state.status_message {
        message_lines.push(Line::from(vec![
            Span::styled(status.clone(), Style::default().fg(state.theme.error_color())),
        ]));
    }

    let messages_widget = Paragraph::new(message_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" hcscoder v{} Chat ", env!("CARGO_PKG_VERSION")))
                .border_style(Style::default().fg(state.theme.border_color())),
        )
        .wrap(Wrap { trim: true })
        .scroll((state.scroll_offset as u16, 0));

    f.render_widget(messages_widget, chunks[0]);

    // Render scrollbar if content is scrollable
    if state.scroll_offset > 0 || message_lines.len() > chunks[0].height as usize {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state = ScrollbarState::new(message_lines.len())
            .position(state.scroll_offset);
        f.render_stateful_widget(
            scrollbar,
            chunks[0],
            &mut scrollbar_state,
        );
    }

    // Input area with theme colors
    let input_title = if state.is_streaming {
        " ⏳ Streaming... (Up/Down to scroll) "
    } else {
        " ✏️  Input (Enter to send, Ctrl+C to quit, ↑↓ scroll) ";
    };

    let input_style = if state.is_streaming {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(state.theme.secondary_color())
    };

    // Add cursor indicator
    let input_text = if state.is_streaming {
        state.input_buffer.clone()
    } else {
        format!("{}▌", state.input_buffer)
    };

    let input_widget = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(input_title)
                .border_style(Style::default().fg(state.theme.border_color())),
        )
        .style(input_style);

    f.render_widget(input_widget, chunks[1]);
}
