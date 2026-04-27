//! OpenAPI Coverage Checker for Jellyfin CLI
//!
//! This tool compares the implemented API endpoints against the Jellyfin OpenAPI spec
//! to generate a coverage report.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Represents an API endpoint
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Endpoint {
    method: String,
    path: String,
    category: String,
}

impl Endpoint {
    fn new(method: &str, path: &str) -> Self {
        let category = extract_category(path);
        Self {
            method: method.to_uppercase(),
            path: path.to_string(),
            category,
        }
    }
}

/// Extract category from path (e.g., /Users/{Id} -> Users)
fn extract_category(path: &str) -> String {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.is_empty() {
        return "Other".to_string();
    }

    let category = parts[0];

    // Map common prefixes to categories
    match category {
        "Users" => "Users".to_string(),
        "Items" => "Items".to_string(),
        "Libraries" => "Library".to_string(),
        "Playback" => "Playback".to_string(),
        "Sessions" => "Sessions".to_string(),
        "System" => "System".to_string(),
        "Plugins" => "Plugins".to_string(),
        "Notifications" => "Notifications".to_string(),
        "ScheduledTasks" => "ScheduledTasks".to_string(),
        "Configuration" => "Configuration".to_string(),
        "Auth" => "Auth".to_string(),
        "Genres" | "Studios" | "Years" | "People" | "Tags" => "Metadata".to_string(),
        "Search" => "Search".to_string(),
        "UserViews" => "UserViews".to_string(),
        "Images" => "Images".to_string(),
        "Videos" => "Videos".to_string(),
        _ => category.to_string(),
    }
}

/// Coverage report
#[derive(Debug)]
struct CoverageReport {
    total_endpoints: usize,
    implemented: usize,
    missing: Vec<Endpoint>,
    by_category: HashMap<String, (usize, usize)>, // category -> (implemented, total)
}

impl CoverageReport {
    fn coverage_percentage(&self) -> f64 {
        if self.total_endpoints == 0 {
            return 0.0;
        }
        (self.implemented as f64 / self.total_endpoints as f64) * 100.0
    }
}

fn main() {
    println!("Jellyfin CLI OpenAPI Coverage Checker\n");

    // Parse OpenAPI spec
    let spec_path = Path::new("openapi/jellyfin-openapi-stable.json");
    if !spec_path.exists() {
        println!("Error: OpenAPI spec not found at {}", spec_path.display());
        println!("Please download the Jellyfin OpenAPI spec and place it at:");
        println!("  openapi/jellyfin-openapi-stable.json");
        std::process::exit(1);
    }

    let spec_content = fs::read_to_string(spec_path).expect("Failed to read OpenAPI spec");
    let spec: serde_json::Value = serde_json::from_str(&spec_content).expect("Failed to parse OpenAPI spec");

    // Extract all endpoints from spec
    let mut spec_endpoints: HashSet<Endpoint> = HashSet::new();

    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (path, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, _) in methods_obj {
                    if ["get", "post", "put", "delete", "patch", "head", "options"]
                        .contains(&method.to_lowercase().as_str())
                    {
                        spec_endpoints.insert(Endpoint::new(method, path));
                    }
                }
            }
        }
    }

    println!("OpenAPI spec loaded: {} endpoints found", spec_endpoints.len());

    // Scan implemented endpoints
    let mut implemented_endpoints: HashSet<Endpoint> = HashSet::new();

    // For now, we'll check the api/src/client.rs file for endpoint patterns
    // This is a simplified approach - in production, you'd parse the code more thoroughly
    let client_code = fs::read_to_string("crates/api/src/client.rs")
        .expect("Failed to read client.rs");

    // Look for common endpoint patterns in the code
    // This is a heuristic approach - parse actual HTTP calls
    for line in client_code.lines() {
        // Look for patterns like self.get_raw("/path") or self.post("/path", ...)
        if let Some(start) = line.find("self.") {
            let after_self = &line[start + 5..];
            let method_name = after_self
                .split(|c: char| c == '(' || c == '<')
                .next()
                .unwrap_or("");

            // Extract path from the line
            if let Some(path_start) = line.find('"') {
                let after_quote = &line[path_start + 1..];
                if let Some(path_end) = after_quote.find('"') {
                    let path = &after_quote[..path_end];
                    if path.starts_with('/') {
                        // Map method names to HTTP methods
                        let http_method = match method_name {
                            "get" | "get_raw" => "GET",
                            "post" | "post_void" => "POST",
                            "put" => "PUT",
                            "delete" => "DELETE",
                            "patch" => "PATCH",
                            _ => "GET",
                        };
                        implemented_endpoints.insert(Endpoint::new(http_method, path));
                    }
                }
            }
        }
    }

    println!("Implemented endpoints found: {}", implemented_endpoints.len());

    // Generate coverage report
    let mut missing: Vec<Endpoint> = spec_endpoints
        .difference(&implemented_endpoints)
        .cloned()
        .collect();
    missing.sort_by(|a, b| a.path.cmp(&b.path).then(a.method.cmp(&b.method)));

    let total = spec_endpoints.len();
    let implemented_count = implemented_endpoints
        .intersection(&spec_endpoints)
        .count();

    // Calculate by category
    let mut by_category: HashMap<String, (usize, usize)> = HashMap::new();

    for endpoint in &spec_endpoints {
        let entry = by_category
            .entry(endpoint.category.clone())
            .or_insert((0, 0));
        entry.1 += 1; // total
    }

    for endpoint in implemented_endpoints.intersection(&spec_endpoints) {
        let entry = by_category
            .entry(endpoint.category.clone())
            .or_insert((0, 0));
        entry.0 += 1; // implemented
    }

    let report = CoverageReport {
        total_endpoints: total,
        implemented: implemented_count,
        missing,
        by_category,
    };

    // Print report
    print_coverage_report(&report);

    // Exit with error if coverage is low (optional)
    if report.coverage_percentage() < 100.0 {
        eprintln!("\nWarning: Coverage is not 100%. Missing {} endpoints.",
            report.total_endpoints - report.implemented);
        std::process::exit(0); // Don't fail for now
    }
}

fn print_coverage_report(report: &CoverageReport) {
    println!("\n" + &"=".repeat(70));
    println!("JELLYFIN CLI OPENAPI COVERAGE REPORT");
    println!("{}", "=".repeat(70));

    println!("\n📊 OVERVIEW");
    println!("  Total Endpoints:      {}", report.total_endpoints);
    println!("  Implemented:          {}", report.implemented);
    println!("  Missing:              {}", report.total_endpoints - report.implemented);
    println!("  Coverage:             {:.1}%", report.coverage_percentage());

    // Coverage bar
    let bar_width = 50;
    let filled = ((report.coverage_percentage() / 100.0) * bar_width as f64) as usize;
    let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);
    println!("  [{}]", bar);

    // By category
    println!("\n📁 COVERAGE BY CATEGORY");
    let mut categories: Vec<_> = report.by_category.iter().collect();
    categories.sort_by(|a, b| b.1 .1.cmp(&a.1 .1)); // Sort by total desc

    for (category, (impl_count, total)) in categories {
        let pct = (*impl_count as f64 / *total as f64) * 100.0;
        let status = if pct >= 100.0 {
            "✅"
        } else if pct >= 50.0 {
            "⚠️"
        } else {
            "❌"
        };
        println!("  {:20} {:>3}/{:<3} ({:>5.1}%) {}",
            category, impl_count, total, pct, status);
    }

    // Missing endpoints (top 20)
    if !report.missing.is_empty() {
        println!("\n⚠️  MISSING ENDPOINTS (showing top 30)");
        for (i, endpoint) in report.missing.iter().take(30).enumerate() {
            println!("  {:>3}. {:6} {}", i + 1, endpoint.method, endpoint.path);
        }
        if report.missing.len() > 30 {
            println!("  ... and {} more", report.missing.len() - 30);
        }
    }

    println!("\n{}", "=".repeat(70));
}
