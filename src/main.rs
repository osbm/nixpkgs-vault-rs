use clap::Parser;
use std::process::Command;
use std::path::Path;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;

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

struct PackageInfo {
    name: String,
    version: String,
    available: bool,
    broken: bool,
    description: Option<String>,
    homepage: Option<String>,
    license_short_name: String,
    long_description: Option<String>,
    maintainers: Vec<String>,
    drv_path: String, // comes from evaluation
    outputs: Vec<String>, // comes from drv file
    input_srcs: Vec<String>, // comes from drv file
    input_drvs: Vec<String>, // comes from drv file
    platforms: Vec<String>,
    dependencies: Vec<String>, // List of dependencies' store paths, comes from the drv file
}


fn main() {
    let args = Args::parse();

    // check if the output directory exists, if not create it
    // if it exists ask the user if they want to overwrite it
    if Path::new(&args.outdir).exists() {
        println!("{} {}", "⚠️  Output directory already exists:".yellow().bold(), args.outdir.bright_white());
        println!("{} {}", "⚠️  Do you want to continue? (y/n): ".yellow().bold(), "");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "y" {
            println!("{}", "❌ Aborting.".red().to_string());
            std::process::exit(1);
        }
    } else {
        std::fs::create_dir_all(&args.outdir).unwrap();
        println!("{} {}", "✅ Created output directory:".green().bold(), args.outdir.bright_white());
    }


    println!(
        "{} {}",
        "📦 Fetching nixpkgs from:".cyan().bold(),
        format!(
            "{}/tree/{}",
            args.git_url.trim_end_matches(".git"),
            args.revision
        ).blue().underline()
    );

    let nixpkgs_path = fetch_nixpkgs_with_nix(&args.git_url, &args.revision);

    println!("{} {}", "✅ Nixpkgs fetched to:".green().bold(), nixpkgs_path.bright_white());

    if !analyze_nixpkgs(&nixpkgs_path) {
        eprintln!("{} {}", "❌ Invalid nixpkgs repository:".red().bold(), nixpkgs_path.bright_white());
        std::process::exit(1);
    }

    let packages_json_path = format!("{}/packages.json", args.outdir);
    if Path::new(&packages_json_path).exists() {
        println!("{} {}", "⚠️  packages.json already exists in:".yellow().bold(), packages_json_path.bright_white());
        println!("{} {}", "⚠️  Skipping computation.".yellow().bold(), "");
    } else {
        // create outdir if not exists
        std::fs::create_dir_all(&args.outdir).unwrap();
        generate_packages_json(&nixpkgs_path, &args.outdir);
    }


    // print loading packages.json
    println!("{} {}", "📥 Loading packages.json to memory:".cyan().bold(), packages_json_path.bright_white());
    // read packages.json
    let package_json_data = std::fs::read_to_string(&packages_json_path).unwrap();

    // parse JSON
    let parsed_json: Value = serde_json::from_str(&package_json_data).unwrap();
    let packages = parsed_json["packages"].as_object().unwrap();

    println!("{} {}", "📊 Total packages found:".cyan().bold(), packages.len().to_string().bright_white());

    // Example: Print first 5 package names with progress bar
    println!("{}", "📦 Processing packages:".cyan().bold());
    let sample_count = packages.len();
    let pb = ProgressBar::new(sample_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );

    for (i, (name, info)) in packages.iter().enumerate() {
        if i >= sample_count {
            break;
        }
        pb.set_message(format!("Processing {}", name));
        let mut package_info = PackageInfo {
            name: name.clone(),
            version: info["version"].as_str().unwrap_or("unknown").to_string(),
            available: info["meta"]["available"].as_bool().unwrap_or(false) == false,
            broken: info["meta"]["broken"].as_bool().unwrap_or(false),
            description: info["meta"]["description"].as_str().map(|s| s.to_string()),
            homepage: info["meta"]["homepage"].as_str().map(|s| s.to_string()),
            license_short_name: info["license"]["shortName"].as_str().unwrap_or("unknown").to_string(),
            long_description: info["meta"]["longDescription"].as_str().map(|s| s.to_string()),
            maintainers: info["meta"]["maintainers"].as_array().map_or(Vec::new(), |arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            drv_path: String::new(),
            outputs: Vec::new(),
            input_srcs: Vec::new(),
            input_drvs: Vec::new(),
            platforms: info["meta"]["platforms"].as_array().map_or(Vec::new(), |arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }),
            dependencies: Vec::new(),
        };

        get_package_info(name, &nixpkgs_path, &mut package_info);

        pb.inc(1);
    }
    pb.finish_with_message("Sample packages loaded!");
    println!();

    println!("{}", "🎉 Done!".green().to_string());
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

fn generate_packages_json(nixpkgs_path: &str, outdir: &str) {
    // nix-env -f . -qa --meta --json --show-trace --arg config 'import ./pkgs/top-level/packages-config.nix' | jq -c '{"version":2,"packages":.}' > packages.json
    // run above command and write it to outdir/packages.json

    let command = format!(
        "nix-env -f {} -qa --meta --json --show-trace --arg config 'import {}/pkgs/top-level/packages-config.nix' | jq -c '{{\"version\":2,\"packages\":.}}' > {}/packages.json",
        nixpkgs_path, nixpkgs_path, outdir
    );

    // with spinner
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    spinner.set_message("Computing packages.json...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .unwrap_or_else(|e| {
            spinner.finish_and_clear();
            eprintln!("{} {}", "❌ Failed to run command:".red().bold(), command.red());
            eprintln!("{} {}", "❌ Error:".red().bold(), format!("Failed to run command: {}", e).red());
            std::process::exit(1);
        });

    if !output.status.success() {
        spinner.finish_and_clear();
        eprintln!("{} {}", "❌ Command failed:".red().bold(), command.red());
        eprintln!("{} {}", "❌ Error:".red().bold(), format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)).red());
        std::process::exit(1);
    }

    spinner.finish_and_clear();
    println!("{}", "✅ packages.json computed successfully!".green().to_string());
}


fn get_package_info(package_name: &str, nixpkgs_path: &str, package_info: &mut PackageInfo) {
    // nix derivation show nixpkgs#package_name
    let command = format!(
        "nix derivation show {}#{}",
        nixpkgs_path, package_name
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(e) => {
            eprintln!("{} {}", "❌ Failed to run command:".red().bold(), command.red());
            eprintln!("{} {}", "❌ Error:".red().bold(), format!("Failed to run command: {}", e).red());
            return;
        }
    };

    if !output.status.success() {
        eprintln!("{} {}", "❌ Command failed:".red().bold(), command.red());
        eprintln!("{} {}", "❌ Error:".red().bold(), format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)).red());
        return;
    }

    let derivation_json = String::from_utf8_lossy(&output.stdout);

    // Parse the JSON output
    if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(&derivation_json) {
        // The output is an object where keys are drv paths
        if let Some(derivation_obj) = parsed_json.as_object() {
            // Get the first (and usually only) derivation
            if let Some((drv_path, drv_data)) = derivation_obj.iter().next() {
                // Set the drv path
                package_info.drv_path = drv_path.clone();

                // Extract outputs
                if let Some(outputs) = drv_data.get("outputs").and_then(|o| o.as_object()) {
                    package_info.outputs = outputs.keys().map(|k| k.clone()).collect();
                }

                // Extract inputDrvs
                if let Some(input_drvs) = drv_data.get("inputDrvs").and_then(|i| i.as_object()) {
                    package_info.input_drvs = input_drvs.keys().map(|k| k.clone()).collect();
                }

                // Extract inputSrcs
                if let Some(input_srcs) = drv_data.get("inputSrcs").and_then(|i| i.as_array()) {
                    package_info.input_srcs = input_srcs.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_string()))
                        .collect();
                }

                // Dependencies are essentially the inputDrvs (store paths of dependencies)
                package_info.dependencies = package_info.input_drvs.clone();
            }
        }
    } else {
        eprintln!("{} Failed to parse derivation JSON for package: {}", "❌".red().bold(), package_name);
    }
}

