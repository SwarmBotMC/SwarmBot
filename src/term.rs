/*
 * Copyright (c) 2021 Andrew Gazelka - All Rights Reserved.
 * Unauthorized copying of this file, via any medium is strictly prohibited.
 * Proprietary and confidential.
 * Written by Andrew Gazelka <andrew.gazelka@gmail.com>, 6/30/21, 1:42 AM
 */

use std::io::stdout;
use std::sync::{Arc, Mutex};

use crossterm::{ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::ClearType;

pub struct Term {
    pub input: std::sync::mpsc::Receiver<String>,
    pub output: std::sync::mpsc::Sender<String>,
    state: Arc<Mutex<String>>,
}

impl Term {
    pub fn init() -> Term {
        let state = Arc::new(Mutex::new(String::new()));

        let mut out = stdout();
        out.queue(crossterm::style::Print("> ")).unwrap();

        let (in_tx, in_rx) = std::sync::mpsc::channel();
        let (out_tx, out_rx) = std::sync::mpsc::channel();

        {
            let state = state.clone();
            rayon::spawn(move ||{
                while let Ok(msg) = out_rx.recv() {
                    let mut out = stdout();
                    out.queue(crossterm::terminal::Clear(ClearType::CurrentLine)).unwrap();

                    out
                        .execute(crossterm::terminal::Clear(ClearType::CurrentLine)).unwrap()
                        .execute(crossterm::cursor::MoveToColumn(0)).unwrap()
                        .execute(crossterm::style::Print(format!("{}\n", msg))).unwrap()
                        .execute(crossterm::style::Print(format!("> {}", state.lock().unwrap()))).unwrap();
                }
            });
        }

        {
            let state = state.clone();
            rayon::spawn(move || {
                while let Ok(event) = crossterm::event::read() {
                    match event {
                        Event::Key(event) => {
                            match event.code {
                                KeyCode::Backspace => {
                                    state.lock().unwrap().pop();
                                }
                                KeyCode::Enter => {
                                    let input: String = state.lock().unwrap().drain(..).collect();
                                    in_tx.send(input).unwrap();
                                    let mut out = stdout();
                                    out.execute(crossterm::style::Print("> ")).unwrap();
                                }
                                KeyCode::Left => {}
                                KeyCode::Right => {}
                                KeyCode::Up => {}
                                KeyCode::Down => {}
                                KeyCode::Home => {}
                                KeyCode::End => {}
                                KeyCode::PageUp => {}
                                KeyCode::PageDown => {}
                                KeyCode::Tab => {}
                                KeyCode::BackTab => {}
                                KeyCode::Delete => {}
                                KeyCode::Insert => {}
                                KeyCode::F(_) => {}
                                KeyCode::Char(c) => {
                                    state.lock().unwrap().push(c);
                                }
                                KeyCode::Null => {}
                                KeyCode::Esc => {}
                            }
                        }
                        Event::Mouse(_) => {}
                        Event::Resize(_, _) => {}
                    }
                }
            });
        }

        Self {
            input: in_rx,
            output: out_tx,
            state,
        }
    }
}
