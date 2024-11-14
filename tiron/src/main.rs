use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Terminal,
};
use std::time::{Duration, Instant};
use std::{io, sync::mpsc};
use std::{str::FromStr, thread};
use tui_input::{backend::crossterm::EventHandler, Input};
use xlsx::{
    base::{expressions::utils::number_to_column, Model},
    export::save_to_xlsx,
    import::load_from_xlsx,
};

use std::env;

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(PartialEq)]
enum CursorMode {
    Navigate,
    Input,
    Popup,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let args: Vec<String> = env::args().collect();
    let mut file_name = "model.xlsx";
    let mut model = if args.len() > 1 {
        file_name = &args[1];
        load_from_xlsx(file_name, "en", "UTC").unwrap()
    } else {
        Model::new_empty(file_name, "en", "UTC").unwrap()
    };
    let mut selected_sheet = 0;
    let mut selected_row_index = 1;
    let mut selected_column_index = 1;
    let mut minimum_row_index = 1;
    let mut minimum_column_index = 1;
    let sheet_list_width = 20;
    let column_width: u16 = 11;
    let mut cursor_mode = CursorMode::Navigate;
    let mut input_formula = Input::default();

    let mut input_file_name: Input = file_name.into();

    let mut popup_open = false;

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let header_style = Style::default().fg(Color::Yellow).bg(Color::White);
    let selected_header_style = Style::default().bg(Color::Yellow).fg(Color::White);

    let selected_cell_style = Style::default().fg(Color::Yellow).bg(Color::LightCyan);

    let background_style = Style::default().bg(Color::Black);
    let selected_sheet_style = Style::default().bg(Color::White).fg(Color::LightMagenta);
    let non_selected_sheet_style = Style::default().fg(Color::White);
    let mut sheet_names = model.workbook.get_worksheet_names();
    loop {
        terminal.draw(|rect| {
            let size = rect.area();

            let global_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(sheet_list_width), Constraint::Min(3)].as_ref())
                .split(size);

            // Sheet list to the left
            let sheets = Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Sheets")
                .border_type(BorderType::Plain)
                .style(background_style);
            let mut rows = vec![];
            (0..sheet_names.len()).for_each(|sheet_index| {
                let sheet_name = &sheet_names[sheet_index];
                let style = if sheet_index == selected_sheet {
                    selected_sheet_style
                } else {
                    non_selected_sheet_style
                };
                rows.push(Row::new(vec![Cell::from(sheet_name.clone()).style(style)]));
            });
            let widths = &[Constraint::Length(100)];
            let sheet_list = Table::new(rows, widths).block(sheets).column_spacing(0);

            rect.render_widget(sheet_list, global_chunks[0]);

            // The spreadsheet is the formula bar at the top and the sheet data
            let spreadsheet_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Length(1), Constraint::Min(2)].as_ref())
                .split(global_chunks[1]);

            let spreadsheet_width = size.width - sheet_list_width;
            let spreadsheet_height = size.height - 1;
            let row_count = spreadsheet_height - 1;

            let first_row_width: u16 = 3;
            let column_count =
                f64::ceil(((spreadsheet_width - first_row_width) as f64) / (column_width as f64))
                    as i32;
            let mut rows = vec![];
            // The first row in the column headers
            let mut row = Vec::new();
            // The first cell in that row is the top left square of the spreadsheet
            row.push(Cell::from(""));
            let mut maximum_column_index = minimum_column_index + column_count - 1;
            let mut maximum_row_index = minimum_row_index + row_count - 1;

            // We want to make sure the selected cell is visible.
            if selected_column_index > maximum_column_index {
                maximum_column_index = selected_column_index;
                minimum_column_index = maximum_column_index - column_count + 1;
            } else if selected_column_index < minimum_column_index {
                minimum_column_index = selected_column_index;
                maximum_column_index = minimum_column_index + column_count - 1;
            }
            if selected_row_index >= maximum_row_index {
                maximum_row_index = selected_row_index;
                minimum_row_index = maximum_row_index - row_count + 1;
            } else if selected_row_index < minimum_row_index {
                minimum_row_index = selected_row_index;
                maximum_row_index = minimum_row_index + row_count - 1;
            }
            for column_index in minimum_column_index..=maximum_column_index {
                let column_str = number_to_column(column_index);
                let style = if column_index == selected_column_index {
                    selected_header_style
                } else {
                    header_style
                };
                row.push(Cell::from(format!("     {}", column_str.unwrap())).style(style));
            }
            rows.push(Row::new(row));
            for row_index in minimum_row_index..=maximum_row_index {
                let mut row = Vec::new();
                let style = if row_index == selected_row_index {
                    selected_header_style
                } else {
                    header_style
                };
                row.push(Cell::from(format!("{}", row_index)).style(style));
                for column_index in minimum_column_index..=maximum_column_index {
                    let value = model
                        .get_formatted_cell_value(
                            selected_sheet as u32,
                            row_index as i32,
                            column_index,
                        )
                        .unwrap();
                    let cell_style = model
                        .get_style_for_cell(selected_sheet as u32, row_index as i32, column_index)
                        .unwrap();
                    let style = if selected_row_index == row_index
                        && selected_column_index == column_index
                    {
                        selected_cell_style
                    } else {
                        let bg_color = match cell_style.fill.fg_color {
                            Some(s) => Color::from_str(&s).unwrap(),
                            None => Color::White,
                        };
                        let fg_color = match cell_style.font.color {
                            Some(s) => Color::from_str(&s).unwrap(),
                            None => Color::Black,
                        };
                        Style::default().fg(fg_color).bg(bg_color)
                    };
                    row.push(Cell::from(value.to_string()).style(style));
                }
                rows.push(Row::new(row));
            }
            let mut widths = Vec::new();
            widths.push(Constraint::Length(first_row_width));
            for _ in 0..column_count {
                widths.push(Constraint::Length(column_width));
            }
            let spreadsheet = Table::new(rows, widths)
                .block(Block::default().style(Style::default().bg(Color::Black)))
                .column_spacing(0);

            let text = if cursor_mode != CursorMode::Input {
                model
                    .get_cell_formula(
                        selected_sheet as u32,
                        selected_row_index as i32,
                        selected_column_index,
                    )
                    .unwrap()
                    .unwrap_or_else(|| {
                        model
                            .get_formatted_cell_value(
                                selected_sheet as u32,
                                selected_row_index as i32,
                                selected_column_index,
                            )
                            .unwrap()
                    })
            } else {
                input_formula.value().to_string()
            };
            let cell_address_text = format!(
                "{}{}: ",
                number_to_column(selected_column_index).unwrap(),
                selected_row_index,
            );
            let formula_bar_text = format!("{}{}", cell_address_text, text,);
            let formula_bar = Paragraph::new(vec![Line::from(vec![Span::raw(formula_bar_text)])]);
            rect.render_widget(formula_bar.block(Block::default()), spreadsheet_chunks[0]);
            rect.render_widget(spreadsheet, spreadsheet_chunks[1]);
            if cursor_mode == CursorMode::Input {
                let area = spreadsheet_chunks[0];
                rect.set_cursor_position((
                    area.x
                        + (input_formula.visual_cursor() as u16)
                        + cell_address_text.len() as u16,
                    area.y,
                ))
            }

            if popup_open {
                let area = centered_rect(60, 20, size);
                rect.render_widget(Clear, area);
                let input_text = input_file_name.value();
                let text = vec![
                    Line::from(vec![input_text.fg(Color::Yellow)]),
                    "".into(),
                    Line::from(vec![
                        "ESC".green(),
                        " to abort. ".into(),
                        "END".green(),
                        " to quit without saving. ".into(),
                        "Enter".green(),
                        " to save and quit".into(),
                    ]),
                ];
                rect.render_widget(
                    Paragraph::new(text).block(Block::bordered().title("Save as")),
                    area,
                );
                rect.set_cursor_position((
                    // Put cursor past the end of the input text
                    area.x + (input_file_name.visual_cursor() as u16) + 1,
                    // Move one line own, from the border to the input line
                    area.y + 1,
                ))
            }
        })?;

        match cursor_mode {
            CursorMode::Popup => {
                match rx.recv()? {
                    Event::Input(event) => match event.code {
                        KeyCode::End => {
                            terminal.clear()?;
                            // restore terminal
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
                            break;
                        }
                        KeyCode::Enter => {
                            terminal.clear()?;
                            // restore terminal
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;
                            let _ = save_to_xlsx(&model, input_file_name.value());
                            break;
                        }
                        KeyCode::Esc => {
                            popup_open = false;
                            cursor_mode = CursorMode::Navigate;
                        }
                        _ => {
                            input_file_name.handle_event(&CEvent::Key(event));
                        }
                    },
                    Event::Tick => {}
                }
            }
            CursorMode::Navigate => {
                match rx.recv()? {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('q') => {
                            popup_open = true;
                            cursor_mode = CursorMode::Popup;
                        }
                        KeyCode::Down => {
                            selected_row_index += 1;
                        }
                        KeyCode::Up => {
                            if selected_row_index > 1 {
                                selected_row_index -= 1;
                            }
                        }
                        KeyCode::Right => {
                            selected_column_index += 1;
                        }
                        KeyCode::Left => {
                            if selected_column_index > 1 {
                                selected_column_index -= 1;
                            }
                        }
                        KeyCode::PageDown => {
                            selected_row_index += 10;
                        }
                        KeyCode::PageUp => {
                            if selected_row_index > 10 {
                                selected_row_index -= 10;
                            } else {
                                selected_row_index = 1;
                            }
                        }
                        KeyCode::Char('s') => {
                            selected_sheet += 1;
                            if selected_sheet >= sheet_names.len() {
                                selected_sheet = 0;
                            }
                        }
                        KeyCode::Char('a') => {
                            selected_sheet = selected_sheet.saturating_sub(1);
                        }
                        KeyCode::Char('e') => {
                            cursor_mode = CursorMode::Input;
                            let input_str = model
                                .get_cell_formula(
                                    selected_sheet as u32,
                                    selected_row_index as i32,
                                    selected_column_index,
                                )
                                .unwrap()
                                .unwrap_or_default();
                            input_formula = input_formula.with_value(input_str);
                        }
                        KeyCode::Char('+') => {
                            model.new_sheet();
                            model.evaluate();
                            sheet_names = model.workbook.get_worksheet_names();
                        }
                        _ => {
                            // println!("{:?}", event);
                        }
                    },
                    Event::Tick => {}
                }
            }
            CursorMode::Input => match rx.recv()? {
                Event::Input(event) => match event.code {
                    // KeyCode::Char(c) => {
                    //     input_str.push(c);
                    // }
                    // KeyCode::Backspace => {
                    //     input_str.pop();
                    // }
                    KeyCode::Enter => {
                        cursor_mode = CursorMode::Navigate;
                        let value = input_formula.value().to_string();
                        let sheet = selected_sheet as i32;
                        let row = selected_row_index as i32;
                        let column = selected_column_index;
                        model
                            .set_user_input(sheet as u32, row, column, value)
                            .unwrap();
                        model.evaluate();
                    }
                    _ => {
                        input_formula.handle_event(&CEvent::Key(event));
                    }
                },
                Event::Tick => {}
            },
        }
    }

    Ok(())
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
