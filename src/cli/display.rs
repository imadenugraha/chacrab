use std::io::IsTerminal;

use console::style;
use serde_json::json;

#[derive(Debug, Clone, Copy)]
pub struct UiOptions {
    pub json: bool,
    pub quiet: bool,
    pub color: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionIndicator {
    Active,
    Locked,
}

pub fn configure_terminal(color: bool) {
    console::set_colors_enabled(color);
}

pub fn is_insecure_terminal() -> bool {
    !std::io::stdout().is_terminal()
}

pub fn print_header(title: &str, state: SessionIndicator, options: UiOptions) {
    if options.quiet || options.json {
        return;
    }

    let state_text = match state {
        SessionIndicator::Active => "[session active]",
        SessionIndicator::Locked => "[locked]",
    };

    println!("chacrab {} {}", env!("CARGO_PKG_VERSION"), state_text);
    println!("---------------------------------------");
    println!("{}", title);
    println!("---------------------------------------");
}

pub fn system(message: &str, options: UiOptions) {
    if options.quiet {
        return;
    }
    if options.json {
        println!("{}", json!({"level":"system", "message": message}));
        return;
    }
    println!("{}", message);
}

pub fn secure(message: &str, options: UiOptions) {
    if options.quiet {
        return;
    }
    if options.json {
        println!("{}", json!({"level":"security", "message": message}));
        return;
    }
    println!("🔐 {}", message);
}

pub fn success(message: &str, options: UiOptions) {
    if options.quiet {
        return;
    }
    if options.json {
        println!("{}", json!({"level":"success", "message": message}));
        return;
    }
    println!("✅ {}", style(message).green());
}

pub fn warning(message: &str, options: UiOptions) {
    if options.quiet {
        return;
    }
    if options.json {
        println!("{}", json!({"level":"warning", "message": message}));
        return;
    }
    println!("⚠️ {}", style(message).yellow());
}

pub fn error(message: &str, options: UiOptions) {
    if options.json {
        println!("{}", json!({"level":"error", "message": message}));
        return;
    }
    eprintln!("⛔ {}", style(message).red());
}

pub fn syncing(message: &str, options: UiOptions) {
    if options.quiet {
        return;
    }
    if options.json {
        println!("{}", json!({"level":"sync", "message": message}));
        return;
    }
    println!("🔄 {}", message);
}

pub fn clear_screen(options: UiOptions) {
    if options.json || options.quiet || is_insecure_terminal() {
        return;
    }
    print!("\x1B[2J\x1B[1;1H");
}

pub fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}
