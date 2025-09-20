use clap::Parser;
use std::process::Command;
use std::path::Path;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// output directory
    #[arg(short, long, default_value = "nixpkgs-vault")]
    outdir: String,

    /// nixpkgs git revision
    #[arg(short, long, default_value = "nixos-unstable")]
    revision: String,

    /// nixpkgs git url
    #[arg(short, long, default_value = "https://github.com/NixOS/nixpkgs.git")]
    git_url: String,

}


fn main() {
    let args = Args::parse();

    println!(
        "{} {}",
        "📦 Fetching nixpkgs from:".cyan().bold(),
        format!(
            "{}/tree/{}",
            args.git_url.trim_end_matches(".git"),
            args.revision
        ).blue().underline()
    );

    // println!(
    //     "{} {}", "⚠️  Warning:".yellow().bold(),
    //     "This may take up to 5 minutes or more depending on your network speed.".yellow());
    let nixpkgs_path = fetch_nixpkgs_with_nix(&args.git_url, &args.revision);

    println!("{} {}", "✅ Nixpkgs fetched to:".green().bold(), nixpkgs_path.bright_white());

    if !analyze_nixpkgs(&nixpkgs_path) {
        eprintln!("{} {}", "❌ Invalid nixpkgs repository:".red().bold(), nixpkgs_path.bright_white());
        std::process::exit(1);
    }



}

fn fetch_nixpkgs_with_nix(git_url: &str, revision: &str) -> String {
    let nix_expr = format!(
        r#"builtins.fetchGit {{ url = "{}"; ref = "{}"; }}"#,
        git_url, revision
    );

    // Create a spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    spinner.set_message("Fetching nixpkgs repository...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let output = Command::new("nix-instantiate")
        .args(&["--eval", "--json", "--expr", &nix_expr])
        .output()
        .unwrap_or_else(|e| {
            spinner.finish_and_clear();
            eprintln!("{}", "❌ Failed to run nix-instantiate".red().to_string());
            eprintln!("{} {}", "❌ Error:".red().bold(), format!("Failed to run nix-instantiate: {}", e).red());
            std::process::exit(1);
        });

    if !output.status.success() {
        spinner.finish_and_clear();
        eprintln!("{}", "❌ nix-instantiate command failed".red().to_string());
        eprintln!("{} {}", "❌ Error:".red().bold(), format!("nix-instantiate failed: {}", String::from_utf8_lossy(&output.stderr)).red());
        std::process::exit(1);
    }

    spinner.finish_and_clear();
    println!("{}", "✅ Repository fetched successfully!".green().to_string());

    let path = String::from_utf8_lossy(&output.stdout)
    .trim()
    .trim_matches('"')
    .to_string();

    path
}

fn analyze_nixpkgs(nixpkgs_path: &str) -> bool {
    // TODO: Implement your analysis logic here, much better than this
    let pkgs_exists = Path::new(&format!("{}/pkgs", nixpkgs_path)).exists();

    pkgs_exists
}
