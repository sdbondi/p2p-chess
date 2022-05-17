use std::{io, io::BufRead, process};

use regex::Regex;

pub struct TorHandle {
    pub child: process::Child,
    pub output: os_pipe::PipeReader,
    pub control_port: u16,
    pub socks_port: u16,
}

impl TorHandle {
    pub fn wait_for_bootstrap(&mut self) -> io::Result<()> {
        let reader = io::BufReader::new(&self.output);
        let regexp = Regex::new(r"Bootstrapped (\d{2})").unwrap();
        let mut lines = String::new();
        for line in reader.lines() {
            let line = line?;
            lines.push_str(&line);
            lines.push('\n');
            if line.contains("[err]") {
                eprintln!("{}", lines);
                self.child.kill()?;
                return Err(io::Error::new(io::ErrorKind::Other, "Tor bootstrap failed"));
            }

            if let Some(caps) = regexp.captures(&line) {
                if let Some(perc) = caps.get(1) {
                    eprintln!("Tor bootstrap {}%", perc.as_str());
                    if perc.as_str().parse::<u8>().unwrap() > 50 {
                        return Ok(());
                    }
                }
            }
        }

        self.child.kill()?;

        eprintln!("{}", lines);
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Tor exited before it could bootstrap",
        ))
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }
}
