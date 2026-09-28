use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use invoice_decoder::DecodedInvoice;

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const DEMO_DELAY_ENV: &str = "LIGHTNING_TUI_DEMO_DELAY_MS";
const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

type DecodeMessage = Result<DecodedInvoice, String>;

#[derive(Default)]
enum DecodeStatus {
    #[default]
    Idle,
    Decoding,
    Success(DecodedInvoice),
    Failed(String),
}

#[derive(Default)]
struct App {
    invoice_input: String,
    status: DecodeStatus,
    decode_receiver: Option<Receiver<DecodeMessage>>,
    result_scroll: u16,
}

impl App {
    fn is_decoding(&self) -> bool {
        matches!(self.status, DecodeStatus::Decoding)
    }

    fn start_decoding(&mut self) {
        if self.is_decoding() {
            return;
        }

        let invoice = self.invoice_input.trim().to_owned();

        if invoice.is_empty() {
            self.status = DecodeStatus::Failed("Enter a BOLT11 invoice first".to_owned());
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.decode_receiver = Some(receiver);
        self.status = DecodeStatus::Decoding;
        self.result_scroll = 0;
        let demo_delay = demo_decode_delay();

        thread::spawn(move || {
            thread::sleep(demo_delay);
            let result = invoice_decoder::decode(&invoice).map_err(|error| error.to_string());
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
    }

    fn scroll_result_down(&mut self, amount: u16) {
        let maximum = match &self.status {
            DecodeStatus::Success(invoice) => advanced_scroll_limit(invoice),
            _ => 0,
        };
        self.result_scroll = self.result_scroll.saturating_add(amount).min(maximum);
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

fn advanced_scroll_limit(invoice: &DecodedInvoice) -> u16 {
    let hop_count: usize = invoice
        .route_hints
        .iter()
        .map(|route| route.hops.len())
        .sum();
    let line_count = 8 + (invoice.route_hints.len() * 2) + (hop_count * 6);
    u16::try_from(line_count).unwrap_or(u16::MAX)
}

pub fn run() -> anyhow::Result<()> {
    let mut terminal = ratatui::try_init().context("failed to initialize the terminal")?;
    let mut app = App::default();
    let result = run_event_loop(&mut terminal, &mut app);
    ratatui::try_restore().context("failed to restore the terminal")?;
    result
}

fn run_event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    loop {
        app.receive_decode_result();

        terminal
            .draw(|frame| draw(frame, app))
            .context("failed to draw the TUI")?;

        if event::poll(Duration::from_millis(250)).context("failed to poll terminal events")?
            && let Event::Key(key) = event::read().context("failed to read a terminal event")?
            && key.kind == KeyEventKind::Press
            && handle_key(app, key)
        {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
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
        KeyCode::Char(character) => {
            app.invoice_input.push(character);
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

fn draw(frame: &mut Frame, app: &App) {
    if let DecodeStatus::Success(invoice) = &app.status {
        draw_result(frame, app, invoice);
    } else {
        draw_input(frame, app);
    }
}

fn draw_input(frame: &mut Frame, app: &App) {
    let [header_area, input_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(7),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let header = Paragraph::new("Lightening Decoder")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::new().borders(Borders::BOTTOM));

    let [editor_area, status_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(3)]).areas(input_area);

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
    let horizontal_scroll = app.invoice_input.len().saturating_sub(visible_width);
    let horizontal_scroll_u16 = u16::try_from(horizontal_scroll).unwrap_or(u16::MAX);

    let status_line = match &app.status {
        DecodeStatus::Idle => Line::styled(
            "Waiting for an invoice",
            Style::default().fg(Color::DarkGray),
        ),
        DecodeStatus::Decoding => {
            let spinner = SPINNER_FRAMES[frame.count() % SPINNER_FRAMES.len()];
            Line::styled(
                format!("{spinner} Decoding invoice..."),
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

    let status = Paragraph::new(status_line).alignment(Alignment::Center);

    let footer = Paragraph::new("Enter  Decode  •  Backspace  Delete  •  Esc / Ctrl+C  Quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(header, header_area);
    frame.render_widget(editor, editor_area);
    frame.render_widget(status, status_area);
    frame.render_widget(footer, footer_area);

    if !app.is_decoding() {
        let cursor_offset =
            u16::try_from(app.invoice_input.len().saturating_sub(horizontal_scroll))
                .unwrap_or(u16::MAX);
        frame.set_cursor_position((editor_area.x + 1 + cursor_offset, editor_area.y + 1));
    }
}

fn draw_result(frame: &mut Frame, app: &App, invoice: &DecodedInvoice) {
    let [header_area, _spacer_area, details_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(8),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let [overview_area, _row_gap, technical_area] = Layout::vertical([
        Constraint::Percentage(45),
        Constraint::Length(1),
        Constraint::Percentage(55),
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

    let header = Paragraph::new("Decoded invoice")
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
    let advanced = advanced_panel(invoice).scroll((app.result_scroll, 0));

    let footer =
        Paragraph::new("↑/↓ or j/k  Scroll advanced  •  n/r  Decode another  •  q/Esc  Quit")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(header, header_area);
    frame.render_widget(summary, summary_area);
    frame.render_widget(validity, validity_area);
    frame.render_widget(identifiers, identifiers_area);
    frame.render_widget(advanced, advanced_area);
    frame.render_widget(footer, footer_area);
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
        Span::styled("Active", Style::default().fg(Color::Green))
    };

    let signature = if invoice.signature_valid {
        Span::styled("Valid", Style::default().fg(Color::Green))
    } else {
        Span::styled("Invalid", Style::default().fg(Color::Red))
    };

    let created_at = format_unix_timestamp(invoice.created_at_unix);
    let expires_at = format_unix_timestamp(invoice.expires_at_unix);

    Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Status     ", Style::default().fg(Color::DarkGray)),
            status,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Signature  ", Style::default().fg(Color::DarkGray)),
            signature,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Created  ", Style::default().fg(Color::DarkGray)),
            Span::raw(created_at),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Expires  ", Style::default().fg(Color::DarkGray)),
            Span::raw(expires_at),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Duration  ", Style::default().fg(Color::DarkGray)),
            Span::raw(format!("{} seconds", invoice.expiry_seconds)),
        ]),
    ])
    .block(panel_block(" Validity "))
}

fn format_unix_timestamp(timestamp: u64) -> String {
    i64::try_from(timestamp)
        .ok()
        .and_then(|value| OffsetDateTime::from_unix_timestamp(value).ok())
        .and_then(|value| value.format(&Rfc3339).ok())
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
        .title(Line::styled(
            title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::DarkGray))
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
    fn formats_unix_timestamps_as_utc() {
        assert_eq!(format_unix_timestamp(0), "1970-01-01T00:00:00Z");
    }
}
