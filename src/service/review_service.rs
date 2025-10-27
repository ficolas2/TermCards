use crate::{
    domain::{card::Card, card_state::ReviewResult, deck::Deck},
    repository::repository::RepositoryError,
};
use atty::Stream;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use libc::c_int;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{self, Write},
    os::unix::io::AsRawFd,
};

use super::service::Service;

const POLL_TIME_MS: c_int = 30;

impl Service {
    pub async fn review(&self, deck_name: String) -> Result<(), RepositoryError> {
        while let Some(card) = self.repository.get_next_card_to_review(&deck_name).await? {
            let result = run_sandboxed_card(&card);
            let mut card_state = self.repository.get_card_state(card.id).await?;
            card_state.apply_review(result);
            self.repository.set_card_state(card_state).await?;
        }

        Ok(())
    }

    pub async fn review_full_deck_by_name(&self, deck_name: String) -> Result<(), RepositoryError> {
        let deck = self.repository.get_deck(&deck_name).await?;
        self.review_full_deck(deck);
        Ok(())
    }

    pub fn review_full_deck(&self, deck: Deck) {
        for card in deck.cards {
            run_sandboxed_card(&card);
        }
    }
}

fn run_sandboxed_card(card: &Card) -> ReviewResult {
    if !atty::is(Stream::Stdin) || !atty::is(Stream::Stdout) {
        eprintln!("TTY required");
        std::process::exit(2);
    }

    print!("\x1b[2J\x1b[H");
    let mut success = false;
    unsafe {
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();

        let mut cmd = CommandBuilder::new("docker");
        cmd.arg("run");
        cmd.arg("-it");
        cmd.arg("--rm");
        cmd.arg("--network=none");
        for (host, cont) in &card.volume_mounts {
            cmd.arg("-v");
            cmd.arg(format!("{host}:{cont}:ro"));
        }
        if let Some(work_dir) = &card.work_dir {
            cmd.arg("-w");
            cmd.arg(work_dir);
        }
        cmd.arg("docker-image");
        if let Some(command) = &card.command {
            cmd.arg("-c");
            cmd.arg(command);
        }

        enable_raw_mode().unwrap();
        let mut child = pair.slave.spawn_command(cmd).unwrap();

        let pty_fd = pair.master.as_raw_fd().unwrap();
        let stdin_fd = io::stdin().as_raw_fd();

        let mut buf = [0u8; 8192];
        let exp = card.expected_output.as_bytes();
        let mut acc: Vec<u8> = Vec::new();

        loop {
            if let Some(_st) = child.try_wait().unwrap() {
                break;
            }

            let mut fds = [
                libc::pollfd {
                    fd: stdin_fd,
                    events: libc::POLLIN,
                    revents: 0,
                },
                libc::pollfd {
                    fd: pty_fd,
                    events: libc::POLLIN,
                    revents: 0,
                },
            ];
            let ret = libc::poll(fds.as_mut_ptr(), 2, POLL_TIME_MS);
            if ret < 0 {
                if *libc::__errno_location() == libc::EINTR {
                    continue;
                } else {
                    break;
                }
            }

            // PTY -> stdout
            if fds[1].revents & libc::POLLIN != 0 {
                let n = libc::read(pty_fd, buf.as_mut_ptr() as *mut _, buf.len());
                if n <= 0 {
                    break;
                }
                io::stdout().write_all(&buf[..n as usize]).unwrap();
                io::stdout().flush().unwrap();
                push_normalized(&mut acc, &buf[..n as usize]);
                if acc.windows(exp.len()).any(|w| w == exp) {
                    success = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                // If acc grows too big, trim all of it except the last match-sized chunk
                if acc.len() > 1 << 20 {
                    let keep = exp.len().saturating_sub(1);
                    let cut = acc.len() - keep;
                    acc.drain(..cut);
                }
            }

            // stdin -> PTY
            if fds[0].revents & libc::POLLIN != 0 {
                let n = libc::read(stdin_fd, buf.as_mut_ptr() as *mut _, buf.len());
                if n > 0 {
                    let _ = libc::write(pty_fd, buf.as_ptr() as *const _, n as usize);
                }
            }
        }
    }
    disable_raw_mode().unwrap();
    let result = if success {
        println!("\n\x1b[1;32mCorrect output!\x1b[0m\x1b[1;32m");
        println!("Expected input was: \x1b[0m{}", card.expected_input);
        println!(
            "\n\x1b[1;31mAgain (1)\x1b[0m  /  \
            \x1b[1;33mHard (2)\x1b[0m  /  \
            \x1b[1;32mGood (3)\x1b[0m  /  \
            \x1b[1;34mEasy (4)\x1b[0m\n"
        );
        enable_raw_mode().unwrap();
        loop {
            if let Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char('1') => break ReviewResult::Again,
                    KeyCode::Char('2') => break ReviewResult::Hard,
                    KeyCode::Char('3') => break ReviewResult::Good,
                    KeyCode::Char('4') => break ReviewResult::Easy,
                    _ => {}
                }
            }
        }
    } else {
        println!(
            "\n\x1b[1;31mCorrect answer was:\x1b[0m {}\n",
            card.expected_input
        );
        event::read().unwrap();
        ReviewResult::Again
    };
    disable_raw_mode().unwrap();

    result
}

fn push_normalized(acc: &mut Vec<u8>, chunk: &[u8]) {
    let mut i = 0;
    while i < chunk.len() {
        let b = chunk[i];
        match b {
            b'\x1b' => {
                // ESC[… ANSI
                i += 1;
                if i < chunk.len() && chunk[i] == b'[' {
                    i += 1;
                    while i < chunk.len() {
                        let c = chunk[i];
                        if (b'@'..=b'~').contains(&c) {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                }
                continue;
            }
            b'\r' => i += 1, // Ignore, only check \n in string match
            b'\x08' => {
                // backspace
                if !acc.is_empty() {
                    acc.pop();
                }
                i += 1;
            }
            _ => {
                acc.push(b);
                i += 1;
            }
        }
    }
}
