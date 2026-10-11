use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use anyhow::Context;
use crossterm::{
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
};
use invoice_decoder::DecodedInvoice;

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};
use time::{OffsetDateTime, format_description};

const DEMO_DELAY_ENV: &str = "LIGHTNING_TUI_DEMO_DELAY_MS";
const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

type DecodeMessage = Result<DecodedInvoice, String>;

#[derive(Clone, Default)]
enum DecodeSource {
    #[default]
    Local,
    Remote(String),
}

impl DecodeSource {
    fn from_api_url(api_url: Option<String>) -> Self {
        api_url.map_or(Self::Local, Self::Remote)
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Local => "Local crate",
            Self::Remote(_) => "Remote API",
        }
    }
}

#[derive(Default)]
enum DecodeStatus {
    #[default]
    Idle,
    Decoding,
    Success(DecodedInvoice),
    Failed(String),
}

#[derive(Clone, Copy)]
enum Dialog {
    Samples,
    Import,
    Export,
}

#[derive(Default)]
struct App {
    invoice_input: String,
    source: DecodeSource,
    status: DecodeStatus,
    decode_receiver: Option<Receiver<DecodeMessage>>,
    result_scroll: u16,
    max_scroll: u16,
    dialog: Option<Dialog>,
    dialog_input: String,
    selected_sample: usize,
    notice: String,
    json_view: bool,
}

impl App {
    fn new(api_url: Option<String>) -> Self {
        Self {
            source: DecodeSource::from_api_url(api_url),
            ..Self::default()
        }
    }

    fn is_decoding(&self) -> bool {
        matches!(self.status, DecodeStatus::Decoding)
    }

    fn start_decoding(&mut self) {
        if self.is_decoding() {
            return;
        }

        self.start_job(None);
    }

    fn start_job(&mut self, image_path: Option<String>) {
        let invoice = crate::files::normalize_invoice(&self.invoice_input).to_owned();

        if invoice.is_empty() && image_path.is_none() {
            self.status = DecodeStatus::Failed("Enter a BOLT11 invoice first".to_owned());
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.decode_receiver = Some(receiver);
        self.status = DecodeStatus::Decoding;
        self.notice.clear();
        self.dialog = None;
        self.result_scroll = 0;
        let demo_delay = demo_decode_delay();
        let source = self.source.clone();

        thread::spawn(move || {
            thread::sleep(demo_delay);
            let result = (|| {
                let invoice = if let Some(path) = image_path {
                    crate::files::read_qr(&path).map_err(|error| format!("{error:#}"))?
                } else {
                    invoice
                };
                match source {
                    DecodeSource::Local => {
                        invoice_decoder::decode(&invoice).map_err(|error| error.to_string())
                    }
                    DecodeSource::Remote(api_url) => {
                        crate::remote::decode(&api_url, &invoice).map_err(|error| error.to_string())
                    }
                }
            })();
            let _ = sender.send(result);
        });
    }

    fn receive_decode_result(&mut self) {
        let Some(receiver) = &self.decode_receiver else {
            return;
        };

        match receiver.try_recv() {
            Ok(Ok(invoice)) => {
                self.status = DecodeStatus::Success(invoice);
                self.decode_receiver = None;
            }
            Ok(Err(error)) => {
                self.status = DecodeStatus::Failed(error);
                self.decode_receiver = None;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.status = DecodeStatus::Failed("Decoder stopped unexpectedly".to_owned());
                self.decode_receiver = None;
            }
        }
    }

    fn clear_result(&mut self) {
        self.status = DecodeStatus::Idle;
    }

    fn has_decoded_invoice(&self) -> bool {
        matches!(self.status, DecodeStatus::Success(_))
    }

    fn reset(&mut self) {
        self.invoice_input.clear();
        self.status = DecodeStatus::Idle;
        self.decode_receiver = None;
        self.result_scroll = 0;
        self.json_view = false;
        self.notice.clear();
        self.dialog = None;
    }

    fn scroll_result_down(&mut self, amount: u16) {
        self.result_scroll = self
            .result_scroll
            .saturating_add(amount)
            .min(self.max_scroll);
    }

    fn scroll_result_up(&mut self, amount: u16) {
        self.result_scroll = self.result_scroll.saturating_sub(amount);
    }
}

fn demo_decode_delay() -> Duration {
    std::env::var(DEMO_DELAY_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(Duration::ZERO, Duration::from_millis)
}

pub fn run(api_url: Option<String>) -> anyhow::Result<()> {
    let mut terminal = ratatui::try_init().context("failed to initialize the terminal")?;
    if let Err(error) = execute!(std::io::stdout(), EnableBracketedPaste) {
        ratatui::restore();
        return Err(error.into());
    }
    let mut app = App::new(api_url);
    let result = run_event_loop(&mut terminal, &mut app);
    let _ = execute!(std::io::stdout(), DisableBracketedPaste);
    ratatui::try_restore().context("failed to restore the terminal")?;
    result
}

fn run_event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    loop {
        app.receive_decode_result();

        terminal
            .draw(|frame| draw(frame, app))
            .context("failed to draw the TUI")?;

        if event::poll(Duration::from_millis(100)).context("failed to poll terminal events")? {
            match event::read().context("failed to read a terminal event")? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if handle_key(app, key) {
                        return Ok(());
                    }
                }
                Event::Paste(text) => handle_paste(app, &text),
                _ => {}
            }
        }
    }
}

fn handle_paste(app: &mut App, text: &str) {
    if app.is_decoding() {
        return;
    }
    let clean: String = text.chars().filter(|c| !c.is_control()).collect();
    if matches!(app.dialog, Some(Dialog::Import | Dialog::Export)) {
        let remaining = 4096_usize.saturating_sub(app.dialog_input.chars().count());
        app.dialog_input.extend(clean.chars().take(remaining));
    } else if app.dialog.is_none() && !app.has_decoded_invoice() {
        let remaining = 8202_usize.saturating_sub(app.invoice_input.chars().count());
        app.invoice_input.extend(clean.chars().take(remaining));
        app.clear_result();
        app.notice.clear();
    }
}

fn open_dialog(app: &mut App, dialog: Dialog) {
    app.dialog = Some(dialog);
    app.notice.clear();
    app.dialog_input = if matches!(dialog, Dialog::Export) {
        if let DecodeStatus::Success(invoice) = &app.status {
            format!(
                "invoice-{}.json",
                &invoice.payment_hash[..12.min(invoice.payment_hash.len())]
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };
}

fn handle_dialog_key(app: &mut App, key: KeyEvent, dialog: Dialog) {
    if key.code == KeyCode::Esc {
        app.dialog = None;
        app.notice.clear();
        return;
    }
    if matches!(dialog, Dialog::Samples) {
        let samples = crate::files::samples();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected_sample = app.selected_sample.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.selected_sample = (app.selected_sample + 1).min(samples.len() - 1);
            }
            KeyCode::Enter => {
                app.invoice_input
                    .clone_from(&samples[app.selected_sample].invoice);
                app.dialog = None;
                app.status = DecodeStatus::Idle;
                "Public demo invoice loaded · expired · press Enter to inspect"
                    .clone_into(&mut app.notice);
            }
            _ => {}
        }
        return;
    }
    match key.code {
        KeyCode::Enter if !app.dialog_input.trim().is_empty() => {
            if matches!(dialog, Dialog::Import) {
                app.start_job(Some(app.dialog_input.clone()));
            } else if let DecodeStatus::Success(invoice) = &app.status {
                match crate::files::export_json(&app.dialog_input, invoice) {
                    Ok(()) => {
                        app.notice = format!("Saved JSON to {}", app.dialog_input);
                        app.dialog = None;
                    }
                    Err(error) => app.notice = format!("{error:#}"),
                }
            }
        }
        KeyCode::Backspace => {
            app.dialog_input.pop();
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.dialog_input.clear();
        }
        KeyCode::Char(c)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            if app.dialog_input.len() < 4096 {
                app.dialog_input.push(c);
            }
        }
        _ => {}
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return true;
    }
    if let Some(dialog) = app.dialog {
        handle_dialog_key(app, key, dialog);
        return false;
    }
    if !app.is_decoding() {
        match key.code {
            KeyCode::F(2) => {
                open_dialog(app, Dialog::Samples);
                return false;
            }
            KeyCode::F(3) => {
                open_dialog(app, Dialog::Import);
                return false;
            }
            KeyCode::Char('e') if app.has_decoded_invoice() => {
                open_dialog(app, Dialog::Export);
                return false;
            }
            KeyCode::Tab if app.has_decoded_invoice() => {
                app.json_view = !app.json_view;
                app.result_scroll = 0;
                return false;
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.reset();
                return false;
            }
            _ => {}
        }
    }
    match key.code {
        KeyCode::Esc => true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => true,
        KeyCode::Char('n' | 'r') if app.has_decoded_invoice() => {
            app.reset();
            false
        }
        KeyCode::Char('q') if app.has_decoded_invoice() => true,
        KeyCode::Down | KeyCode::Char('j') if app.has_decoded_invoice() => {
            app.scroll_result_down(1);
            false
        }
        KeyCode::PageDown if app.has_decoded_invoice() => {
            app.scroll_result_down(5);
            false
        }
        KeyCode::Up | KeyCode::Char('k') if app.has_decoded_invoice() => {
            app.scroll_result_up(1);
            false
        }
        KeyCode::PageUp if app.has_decoded_invoice() => {
            app.scroll_result_up(5);
            false
        }
        KeyCode::Home if app.has_decoded_invoice() => {
            app.result_scroll = 0;
            false
        }
        _ if app.has_decoded_invoice() => false,
        KeyCode::Enter => {
            app.start_decoding();
            false
        }
        KeyCode::Char(_) | KeyCode::Backspace if app.is_decoding() => false,
        KeyCode::Char(character)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            if app.invoice_input.len() < 8202 {
                app.invoice_input.push(character);
            }
            app.notice.clear();
            app.clear_result();
            false
        }
        KeyCode::Backspace => {
            app.invoice_input.pop();
            app.clear_result();
            false
        }
        _ => false,
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    if frame.area().width < 72 || frame.area().height < 24 {
        frame.render_widget(Paragraph::new("Lightening Decoder\n\nPlease resize to at least 72 × 24 for the inspector.\nYour current input is preserved. Ctrl+C quits.")
            .wrap(Wrap { trim: false }).block(panel_block(" Resize terminal ")), frame.area());
        return;
    }
    if let DecodeStatus::Success(invoice) = &app.status {
        let (scroll, maximum) = draw_result(frame, app, invoice);
        app.result_scroll = scroll;
        app.max_scroll = maximum;
    } else {
        draw_input(frame, app);
    }
    if let Some(dialog) = app.dialog {
        draw_dialog(frame, app, dialog);
    }
}

fn input_header(app: &App) -> Paragraph<'_> {
    Paragraph::new(vec![
        Line::from(""),
        Line::styled(
            "ϟ  LIGHTENING / INVOICE INSPECTOR",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::styled(
            "Paste an invoice. Drop into the details.",
            Style::default().fg(Color::White),
        ),
        Line::styled(
            format!(
                "{} · BOLT11 · no payments, just clarity",
                app.source.label()
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ])
    .alignment(Alignment::Center)
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::new().borders(Borders::BOTTOM))
}

fn draw_input_actions(frame: &mut Frame, actions_area: Rect) {
    let [sample_area, image_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .areas(actions_area);
    frame.render_widget(Paragraph::new("\nExplore public examples\nCoffee payment · Open donation\nExpired fixtures, ready to inspect")
        .alignment(Alignment::Center).block(panel_block(" F2 · Sample gallery ")), sample_area);
    frame.render_widget(
        Paragraph::new(
            "\nRead a QR from your files\nPNG · JPEG · WebP\nImage processing stays on this device",
        )
        .alignment(Alignment::Center)
        .block(panel_block(" F3 · Import QR image ")),
        image_area,
    );
}

fn draw_input(frame: &mut Frame, app: &App) {
    let [header_area, input_area, footer_area] = Layout::vertical([
        Constraint::Length(7),
        Constraint::Min(7),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let header = input_header(app);

    let [editor_area, status_area, actions_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Min(3),
    ])
    .areas(input_area);

    let editor_line = if app.invoice_input.is_empty() {
        Line::styled(
            " Type or paste a BOLT11 invoice",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )
    } else {
        Line::from(app.invoice_input.as_str())
    };

    let visible_width = editor_area.width.saturating_sub(3) as usize;
    let horizontal_scroll = app
        .invoice_input
        .chars()
        .count()
        .saturating_sub(visible_width);
    let horizontal_scroll_u16 = u16::try_from(horizontal_scroll).unwrap_or(u16::MAX);

    let status_line = match &app.status {
        DecodeStatus::Idle => Line::styled(
            format!("Waiting for an invoice · {}", app.source.label()),
            Style::default().fg(Color::DarkGray),
        ),
        DecodeStatus::Decoding => {
            let spinner = SPINNER_FRAMES[frame.count() % SPINNER_FRAMES.len()];
            Line::styled(
                format!("{spinner} Decoding invoice via {}...", app.source.label()),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        }
        DecodeStatus::Success(invoice) => Line::styled(
            format!("Decoded {} invoice", invoice.network),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        DecodeStatus::Failed(error) => {
            Line::styled(error.as_str(), Style::default().fg(Color::Red))
        }
    };

    let editor = Paragraph::new(editor_line)
        .scroll((0, horizontal_scroll_u16))
        .block(
            Block::bordered()
                .title(Line::styled(
                    " BOLT11 invoice ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    let status = Paragraph::new(vec![
        Line::from(""),
        status_line,
        Line::styled(app.notice.as_str(), Style::default().fg(Color::Cyan)),
    ])
    .alignment(Alignment::Center);
    draw_input_actions(frame, actions_area);

    let footer =
        Paragraph::new("Enter Decode  •  F2 Samples  •  F3 QR image  •  Ctrl+U Clear  •  Esc Quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(header, header_area);
    frame.render_widget(editor, editor_area);
    frame.render_widget(status, status_area);
    frame.render_widget(footer, footer_area);

    if !app.is_decoding() {
        let cursor_offset = u16::try_from(
            app.invoice_input
                .chars()
                .count()
                .saturating_sub(horizontal_scroll),
        )
        .unwrap_or(u16::MAX);
        frame.set_cursor_position((editor_area.x + 1 + cursor_offset, editor_area.y + 1));
    }
}

fn draw_result(frame: &mut Frame, app: &App, invoice: &DecodedInvoice) -> (u16, u16) {
    let [header_area, _spacer_area, details_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let [overview_area, _row_gap, technical_area] = Layout::vertical([
        Constraint::Length(9),
        Constraint::Length(1),
        Constraint::Min(5),
    ])
    .areas(details_area);

    let [summary_area, _summary_gap, validity_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(overview_area);

    let [identifiers_area, _identifiers_gap, advanced_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(technical_area);

    let header = Paragraph::new(format!("Decoded invoice · {}", app.source.label()))
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::new().borders(Borders::BOTTOM));

    let summary = summary_panel(invoice);
    let validity = validity_panel(invoice);
    let identifiers = identifiers_panel(invoice);
    let advanced = advanced_panel(invoice);

    let footer = Paragraph::new(vec![
        Line::styled(app.notice.as_str(), Style::default().fg(Color::Cyan)),
        Line::from(
            "Tab JSON/Details · e Export · ↑/↓ Scroll · F2 Samples · F3 QR · n New · q Quit",
        ),
    ])
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(header, header_area);
    let maximum;
    if app.json_view {
        let json = Paragraph::new(serde_json::to_string_pretty(invoice).unwrap_or_default())
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Cyan))
            .block(panel_block(" JSON · e to export "));
        maximum = scroll_limit(&json, details_area);
        frame.render_widget(
            json.scroll((app.result_scroll.min(maximum), 0)),
            details_area,
        );
    } else {
        let panels = [
            (summary, summary_area),
            (validity, validity_area),
            (identifiers, identifiers_area),
            (advanced, advanced_area),
        ];
        maximum = panels
            .iter()
            .map(|(panel, area)| scroll_limit(panel, *area))
            .max()
            .unwrap_or(0);
        for (panel, area) in panels {
            let scroll = app.result_scroll.min(scroll_limit(&panel, area));
            frame.render_widget(panel.scroll((scroll, 0)), area);
        }
    }
    frame.render_widget(footer, footer_area);
    (app.result_scroll.min(maximum), maximum)
}

fn scroll_limit(paragraph: &Paragraph<'_>, area: Rect) -> u16 {
    u16::try_from(
        paragraph
            .line_count(area.width)
            .saturating_sub(usize::from(area.height)),
    )
    .unwrap_or(u16::MAX)
}

fn summary_panel(invoice: &DecodedInvoice) -> Paragraph<'_> {
    let amount = invoice.amount_msat.map_or_else(
        || "Not specified".to_owned(),
        |value| format!("{value} msat"),
    );
    let description = invoice.description.as_deref().unwrap_or("Not included");

    Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Network  ", Style::default().fg(Color::DarkGray)),
            Span::raw(invoice.network.as_str()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Amount   ", Style::default().fg(Color::DarkGray)),
            Span::raw(amount),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Description  ", Style::default().fg(Color::DarkGray)),
            Span::raw(description),
        ]),
    ])
    .wrap(Wrap { trim: false })
    .block(panel_block(" Summary "))
}

fn validity_panel(invoice: &DecodedInvoice) -> Paragraph<'static> {
    let status = if invoice.expired {
        Span::styled("Expired", Style::default().fg(Color::Red))
    } else {
        Span::styled("Not expired", Style::default().fg(Color::Green))
    };

    let signature = if invoice.signature_valid {
        Span::styled("Valid", Style::default().fg(Color::Green))
    } else {
        Span::styled("Invalid", Style::default().fg(Color::Red))
    };

    let created_at = format_unix_timestamp(invoice.created_at_unix);
    let expires_at = format_unix_timestamp(invoice.expires_at_unix);

    Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Status     ", Style::default().fg(Color::DarkGray)),
            status,
        ]),
        Line::from(vec![
            Span::styled("Signature  ", Style::default().fg(Color::DarkGray)),
            signature,
        ]),
        Line::from(vec![
            Span::styled("Created  ", Style::default().fg(Color::DarkGray)),
            Span::raw(created_at),
        ]),
        Line::from(vec![
            Span::styled("Expires  ", Style::default().fg(Color::DarkGray)),
            Span::raw(expires_at),
        ]),
        Line::from(vec![
            Span::styled("Duration  ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{} seconds", invoice.expiry_seconds)),
        ]),
    ])
    .wrap(Wrap { trim: false })
    .block(panel_block(" Validity "))
}

fn format_unix_timestamp(timestamp: u64) -> String {
    let format = format_description::parse_borrowed::<2>(
        "[day padding:none] [month repr:short] [year], [hour]:[minute] UTC",
    )
    .expect("the timestamp display format should be valid");
    i64::try_from(timestamp)
        .ok()
        .and_then(|value| OffsetDateTime::from_unix_timestamp(value).ok())
        .and_then(|value| value.format(&format).ok())
        .unwrap_or_else(|| format!("{timestamp} (Unix)"))
}

fn identifiers_panel(invoice: &DecodedInvoice) -> Paragraph<'_> {
    Paragraph::new(vec![
        Line::from(""),
        Line::styled("Payment hash", Style::default().fg(Color::DarkGray)),
        Line::from(invoice.payment_hash.as_str()),
        Line::from(""),
        Line::styled("Payee public key", Style::default().fg(Color::DarkGray)),
        Line::from(invoice.payee_public_key.as_str()),
    ])
    .wrap(Wrap { trim: false })
    .block(panel_block(" Payment identifiers "))
}

fn advanced_panel(invoice: &DecodedInvoice) -> Paragraph<'_> {
    let description_hash = invoice
        .description_hash
        .as_deref()
        .unwrap_or("Not included");
    let fallback_addresses = if invoice.fallback_addresses.is_empty() {
        "None".to_owned()
    } else {
        invoice.fallback_addresses.join(", ")
    };
    let route_hop_count: usize = invoice
        .route_hints
        .iter()
        .map(|route| route.hops.len())
        .sum();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Description hash  ", Style::default().fg(Color::DarkGray)),
            Span::raw(description_hash),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Minimum final CLTV  ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{} blocks", invoice.min_final_cltv_expiry_delta)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Fallback addresses  ", Style::default().fg(Color::DarkGray)),
            Span::raw(fallback_addresses),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Route hints  ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!(
                "{} routes, {route_hop_count} hops",
                invoice.route_hints.len()
            )),
        ]),
    ];
    lines.extend(route_hint_lines(invoice));

    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(panel_block(" Advanced "))
}

fn route_hint_lines(invoice: &DecodedInvoice) -> Vec<Line<'_>> {
    let mut lines = Vec::new();

    for (route_index, route) in invoice.route_hints.iter().enumerate() {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            format!("Route {}", route_index + 1),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

        for (hop_index, hop) in route.hops.iter().enumerate() {
            let minimum = hop
                .htlc_minimum_msat
                .map_or_else(|| "None".to_owned(), |value| value.to_string());
            let maximum = hop
                .htlc_maximum_msat
                .map_or_else(|| "None".to_owned(), |value| value.to_string());

            lines.push(Line::styled(
                format!("  Hop {}", hop_index + 1),
                Style::default().add_modifier(Modifier::BOLD),
            ));
            lines.push(Line::from(format!(
                "    Source node: {}",
                hop.source_node_id
            )));
            lines.push(Line::from(format!(
                "    Short channel ID: {}",
                hop.short_channel_id
            )));
            lines.push(Line::from(format!(
                "    Fees: {} msat base + {} ppm",
                hop.base_fee_msat, hop.proportional_fee_millionths
            )));
            lines.push(Line::from(format!(
                "    CLTV delta: {} blocks",
                hop.cltv_expiry_delta
            )));
            lines.push(Line::from(format!(
                "    HTLC range: {minimum}–{maximum} msat"
            )));
        }
    }

    lines
}

fn panel_block(title: &'static str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(Line::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::DarkGray))
}

fn draw_dialog(frame: &mut Frame, app: &App, dialog: Dialog) {
    let area = frame.area();
    let width = area.width.saturating_sub(8).min(86);
    let height = 14.min(area.height);
    let rect = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, rect);
    let title = match dialog {
        Dialog::Samples => " Sample gallery ",
        Dialog::Import => " Import QR image ",
        Dialog::Export => " Export JSON ",
    };
    let inner = rect.inner(ratatui::layout::Margin::new(2, 1));
    frame.render_widget(
        panel_block(title)
            .border_style(Style::default().fg(Color::Yellow))
            .style(Style::default().bg(Color::Black)),
        rect,
    );
    if matches!(dialog, Dialog::Samples) {
        let mut lines = vec![
            Line::styled(
                "Public BOLT11 examples · expired · inspection only",
                Style::default().fg(Color::DarkGray),
            ),
            Line::from(""),
        ];
        for (index, sample) in crate::files::samples().iter().enumerate() {
            let selected = index == app.selected_sample;
            lines.push(Line::styled(
                format!("{} {}", if selected { "›" } else { " " }, sample.label),
                if selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ));
            lines.push(Line::styled(
                format!("  {}", sample.detail),
                Style::default().fg(Color::DarkGray),
            ));
            lines.push(Line::from(""));
        }
        lines.push(Line::from("↑/↓ Select   Enter Load   Esc Cancel"));
        frame.render_widget(Paragraph::new(lines), inner);
    } else {
        let [help, field, status] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .areas(inner);
        let hint = if matches!(dialog, Dialog::Import) {
            "Paste an image path (quotes and ~/ supported).\nPNG / JPEG / WebP · up to 10 MB and 4096 × 4096"
        } else {
            "Choose a destination for the decoded invoice.\nExisting files are never overwritten."
        };
        frame.render_widget(
            Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
            help,
        );
        let visible = usize::from(field.width.saturating_sub(3));
        let scroll = app.dialog_input.chars().count().saturating_sub(visible);
        frame.render_widget(
            Paragraph::new(app.dialog_input.as_str())
                .scroll((0, u16::try_from(scroll).unwrap_or(u16::MAX)))
                .block(panel_block(" File path ")),
            field,
        );
        frame.render_widget(
            Paragraph::new(format!(
                "Enter Continue · Ctrl+U Clear · Esc Cancel\n{}",
                app.notice
            ))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Yellow)),
            status,
        );
        let offset = app.dialog_input.chars().count().min(visible);
        frame.set_cursor_position((
            field.x + 1 + u16::try_from(offset).unwrap_or(0),
            field.y + 1,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_invoice() -> DecodedInvoice {
        DecodedInvoice {
            network: "regtest".to_owned(),
            amount_msat: Some(1_000),
            description: Some("Test payment".to_owned()),
            description_hash: None,
            payment_hash: "payment-hash".to_owned(),
            payee_public_key: "payee-key".to_owned(),
            created_at_unix: 0,
            expires_at_unix: 3_600,
            expiry_seconds: 3_600,
            expired: false,
            signature_valid: true,
            min_final_cltv_expiry_delta: 18,
            fallback_addresses: Vec::new(),
            route_hints: Vec::new(),
        }
    }

    #[test]
    fn characters_and_backspace_update_the_invoice_input() {
        let mut app = App::default();

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
        );
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
        );
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );

        assert_eq!(app.invoice_input, "l");
        assert!(matches!(app.status, DecodeStatus::Idle));
    }

    #[test]
    fn enter_rejects_empty_input() {
        let mut app = App::default();

        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(matches!(app.status, DecodeStatus::Failed(_)));
    }

    #[test]
    fn receives_a_successful_background_result() {
        let mut app = App {
            status: DecodeStatus::Decoding,
            ..App::default()
        };
        let (sender, receiver) = mpsc::channel();
        app.decode_receiver = Some(receiver);
        sender
            .send(Ok(sample_invoice()))
            .expect("the test receiver should remain connected");

        app.receive_decode_result();

        assert!(matches!(app.status, DecodeStatus::Success(_)));
        assert!(app.decode_receiver.is_none());
    }

    #[test]
    fn result_keys_scroll_and_reset_the_screen() {
        let mut app = App {
            invoice_input: "invoice".to_owned(),
            max_scroll: 10,
            status: DecodeStatus::Success(sample_invoice()),
            ..App::default()
        };

        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.result_scroll, 1);

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
        );
        assert!(app.invoice_input.is_empty());
        assert_eq!(app.result_scroll, 0);
        assert!(matches!(app.status, DecodeStatus::Idle));
    }

    #[test]
    fn sample_and_import_dialogs_support_selection_paste_and_cancel() {
        let mut app = App::default();
        handle_key(&mut app, KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE));
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.invoice_input, crate::files::samples()[1].invoice);
        assert!(app.dialog.is_none());
        handle_key(&mut app, KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
        handle_paste(&mut app, "'/tmp/invoice image.png'");
        assert_eq!(app.dialog_input, "'/tmp/invoice image.png'");
        assert!(!handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
        ));
        assert!(app.dialog.is_none());
        assert_eq!(app.invoice_input, crate::files::samples()[1].invoice);
    }

    #[test]
    fn paste_is_bounded_and_does_not_trigger_shortcuts() {
        let mut app = App::default();
        handle_paste(&mut app, "lightning:lnbc\r\n");
        assert_eq!(app.invoice_input, "lightning:lnbc");
        handle_paste(&mut app, &"x".repeat(9000));
        assert_eq!(app.invoice_input.len(), 8202);
        app.status = DecodeStatus::Decoding;
        handle_paste(&mut app, "ignored");
        assert_eq!(app.invoice_input.len(), 8202);
    }

    #[test]
    fn json_view_and_export_dialog_preserve_results() {
        let mut app = App {
            status: DecodeStatus::Success(sample_invoice()),
            ..App::default()
        };
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(app.json_view);
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
        );
        assert!(matches!(app.dialog, Some(Dialog::Export)));
        handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.has_decoded_invoice());
    }

    #[test]
    fn renders_gallery_results_json_and_small_terminal_without_panics() {
        use ratatui::{Terminal, backend::TestBackend};
        let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
        let mut app = App::default();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let screen = format!("{:?}", terminal.backend().buffer());
        assert!(screen.contains("INVOICE INSPECTOR"));
        assert!(screen.contains("Sample gallery"));
        open_dialog(&mut app, Dialog::Samples);
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(format!("{:?}", terminal.backend().buffer()).contains("Open donation"));
        app.dialog = None;
        app.status = DecodeStatus::Success(sample_invoice());
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(format!("{:?}", terminal.backend().buffer()).contains("Payment identifiers"));
        app.json_view = true;
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(format!("{:?}", terminal.backend().buffer()).contains("amount_msat"));
        let mut small = Terminal::new(TestBackend::new(40, 12)).unwrap();
        small.draw(|frame| draw(frame, &mut app)).unwrap();
        assert!(format!("{:?}", small.backend().buffer()).contains("Resize terminal"));
    }

    #[test]
    fn formats_unix_timestamps_as_utc() {
        assert_eq!(format_unix_timestamp(0), "1 Jan 1970, 00:00 UTC");
    }

    #[test]
    fn selects_the_remote_source_when_an_api_url_is_present() {
        let app = App::new(Some("http://localhost:3001".to_owned()));

        assert!(matches!(app.source, DecodeSource::Remote(_)));
        assert_eq!(app.source.label(), "Remote API");
    }
}
