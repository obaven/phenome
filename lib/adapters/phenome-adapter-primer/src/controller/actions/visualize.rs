use anyhow::{Context, Result};
use tracing::info;

use super::generate::StorageArgs;
use bootstrappo::ports::visualizer::{OutputFormat, VisualizerPort};

pub async fn generate_storage(
    args: StorageArgs,
    format: String,
    layout: String,
    output: Option<String>,
) -> Result<()> {
    info!("Scanning for storage devices to visualize...");
    let devices =
        bootstrappo::adapters::infrastructure::kube::discovery::storage::scan_block_devices(
            args.min_size,
        )?;

    if devices.is_empty() {
        info!("No storage devices found to visualize.");
        // We might still want to show apps if they are configured, but generally we need devices.
    }

    // Load Config to get App Assignments
    // We try to load, but if it fails (no file), we warn and verify only physical layout
    if let Err(e) = bootstrappo::application::config::load() {
        tracing::warn!(
            "Failed to load config for app assignment optimization: {}",
            e
        );
    }
    // We can assume load() might have worked or we have default.
    // If load failed, instance() panics or returns default.
    // Ideally bootstrappo::application::config should be safe.
    // Let's assume for CLI usage, load failure is bad, but we might be running against no config.
    // Safe guard:
    let config_arc = if bootstrappo::application::config::instance_exists() {
        Some(bootstrappo::application::config::instance())
    } else {
        None
    };

    // Generate DOT
    let mut dot = String::new();
    dot.push_str("digraph system_storage {\n");
    dot.push_str(&format!("  layout=\"{layout}\";\n"));
    dot.push_str("  rankdir=\"LR\";\n"); // Left-to-Right matches App -> Disk better
    dot.push_str("  node [fontname=\"Times-New-Roman\", shape=note];\n");
    dot.push_str("  edge [fontname=\"Times-New-Roman\", fontsize=10];\n");
    dot.push_str("  labelloc=\"t\";\n");
    dot.push_str("  label=\"System Storage Topology\";\n");

    // 1. Render Devices (Physical Layer)
    dot.push_str("  subgraph cluster_physical {\n");
    dot.push_str("    label = \"Physical Devices\";\n");
    dot.push_str("    style = filled;\n");
    dot.push_str("    color = \"#f5f5f5\";\n");

    for dev in devices.iter() {
        let safe_dev_name = dev.name.replace("-", "_");
        let node_id = format!("dev_{safe_dev_name}");

        let size_info = match dev.size_gb {
            Some(s) => format!("<TR><TD><FONT POINT-SIZE=\"10\">Capacity: {s}Gi</FONT></TD></TR>"),
            None => "".to_string(),
        };

        // HTML-like label for structured device info
        let label = format!(
            "<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">\
             <TR><TD BGCOLOR=\"#0288d1\"><FONT COLOR=\"white\"><B>{name_upper}</B></FONT></TD></TR>\
             <TR><TD>{path}</TD></TR>\
             {size_info}\
             <TR><TD><FONT POINT-SIZE=\"10\">Type: {device_type:?}</FONT></TD></TR>\
             <TR><TD><FONT POINT-SIZE=\"10\">Tier: {performance:?}</FONT></TD></TR>\
             </TABLE>>",
            name_upper = dev.name.to_uppercase(),
            path = dev.path,
            size_info = size_info,
            device_type = dev.device_type,
            performance = dev.performance
        );

        dot.push_str(&format!(
            "    {node_id} [label={label}, style=\"filled\", fillcolor=\"#ffffff:#e1f5fe\", gradientangle=90, shape=cylinder, tooltip=\"Device: {device} - Path: {path}\"];\n",
            device = dev.name,
            path = dev.path
        ));
    }
    dot.push_str("  }\n");

    // 2. Render Apps (Logical Layer)
    if let Some(config) = config_arc {
        use bootstrappo::application::runtime::registry;
        use bootstrappo::ports::module::{ModuleContext, ModuleMode};
        use std::collections::{BTreeMap, HashMap};
        use std::sync::Arc;

        let mut path_nodes = Vec::new();
        let mut usage_per_path: HashMap<String, u32> = HashMap::new();

        // Pre-populate path IDs
        let paths = &config.storage.local_path.paths;
        for (i, p) in paths.iter().enumerate() {
            let path_id = format!("path_{i}");
            path_nodes.push((path_id, p.clone()));
        }

        // Helper to find target node (Path or Device) and its base path
        let resolve_target =
            |profile: &str, path_nodes: &[(String, String)]| -> (String, Option<String>) {
                // Priority 0: Map "balanced" or "default" to "bulk" to avoid "virtual_default"
                let effective_profile = match profile {
                    "balanced" | "default" => "bulk",
                    other => other,
                };

                // Priority 1: Match a configured Host Path by name (e.g. profile "bulk" -> path "/mnt/storage-bulk")
                for (id, p) in path_nodes {
                    // Heuristic: if path contains profile name
                    if p.contains(effective_profile) {
                        return (id.clone(), Some(p.clone()));
                    }
                }

                // Priority 2: Direct Device Match (if no path layer match)
                if devices.iter().any(|d| d.name == effective_profile) {
                    let safe_profile = effective_profile.replace("-", "_");
                    return (format!("dev_{safe_profile}"), None);
                }

                // Priority 3: Fallback "virtual" path if profile is used but not mapped
                ("virtual_default".to_string(), None)
            };

        // ---------------------------------------------------------
        // 2a. Discover Apps & Calculate usage
        // ---------------------------------------------------------
        let ctx = ModuleContext::new(Arc::clone(&config), ModuleMode::Render);
        let modules = registry::get_all_modules(config.as_ref());

        // Grouping by domain: (name, domain, storage_gib, resolve_res, type)
        type StorageGroup = (String, String, u32, (String, Option<String>), String);
        type StorageGroups = BTreeMap<String, Vec<StorageGroup>>;

        let mut groups: StorageGroups = BTreeMap::new();

        for module in modules {
            let spec = module.spec();
            let name = spec.name.to_string();
            let domain = spec.domain.to_string();

            if module.enabled(&ctx) {
                let r = config.resources.for_app(&name);
                if r.storage_gib > 0 {
                    let profile = r.storage_profile.as_deref().unwrap_or("default");
                    let res = resolve_target(profile, &path_nodes);

                    // Track usage per path
                    if res.0.starts_with("path_") {
                        *usage_per_path.entry(res.0.clone()).or_default() += r.storage_gib;
                    }

                    // Categorize as Fast or Bulk for label statistics
                    let storage_type = if profile.contains("fast") {
                        "fast"
                    } else {
                        "bulk"
                    };

                    groups.entry(domain.clone()).or_default().push((
                        name.to_string(),
                        domain.to_string(),
                        r.storage_gib,
                        res,
                        storage_type.to_string(),
                    ));
                }
            }
        }

        // ---------------------------------------------------------
        // 2b. Render Host Paths (Logical Storage) with Stats
        // ---------------------------------------------------------
        dot.push_str("  subgraph cluster_paths {\n");
        dot.push_str("    label = \"Host Paths (PVC Binding)\";\n");
        dot.push_str("    style = filled;\n");
        dot.push_str("    color = \"#fff3e0\";\n"); // Orange tint

        for (path_id, p) in &path_nodes {
            let usage = usage_per_path.get(path_id).cloned().unwrap_or(0);

            // Resolve capacity by finding the device prefix
            let mut capacity_gb = 0;
            for dev in &devices {
                if let Some(mnt) = &dev.mountpoint {
                    if p.starts_with(mnt) {
                        capacity_gb = dev.size_gb.unwrap_or(0);
                        break;
                    }
                }
            }

            let remaining = capacity_gb.saturating_sub(usage as u64);

            let label = format!(
                "<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">\
                 <TR><TD BGCOLOR=\"#ef6c00\"><FONT COLOR=\"white\"><B>{p}</B></FONT></TD></TR>\
                 <TR><TD ALIGN=\"LEFT\"><FONT POINT-SIZE=\"10\">Capacity: {capacity_gb}Gi</FONT></TD></TR>\
                 <TR><TD ALIGN=\"LEFT\"><FONT COLOR=\"#616161\" POINT-SIZE=\"10\">Allocated: {usage}Gi</FONT></TD></TR>\
                 <TR><TD ALIGN=\"LEFT\"><FONT COLOR=\"#1b5e20\" POINT-SIZE=\"10\"><B>Remaining: {remaining}Gi</B></FONT></TD></TR>\
                 </TABLE>>"
            );

            dot.push_str(&format!(
                "    {path_id} [label={label}, shape=folder, style=filled, fillcolor=\"#ffe0b2\"];\n"
            ));
        }
        dot.push_str("  }\n");

        // ---------------------------------------------------------
        // 2c. Render Apps (Assignments)
        // ---------------------------------------------------------
        dot.push_str("  subgraph cluster_apps {\n");
        dot.push_str("    label = \"Application Assignments\";\n");
        dot.push_str("    style = dashed;\n");
        dot.push_str("    color = \"#eeeeee\";\n");

        // Render groups
        for (domain, apps) in groups {
            let fast_sum: u32 = apps
                .iter()
                .filter(|(_, _, _, _, t)| t == "fast")
                .map(|(_, _, s, _, _)| *s)
                .sum();
            let bulk_sum: u32 = apps
                .iter()
                .filter(|(_, _, _, _, t)| t == "bulk")
                .map(|(_, _, s, _, _)| *s)
                .sum();

            let safe_domain = domain.replace("-", "_");
            let domain_upper = domain.to_uppercase();

            // Map domains to professional color pairs (Header, ClusterBG)
            let (header_color, bg_color) = match domain_upper.as_str() {
                "ANALYTICS" => ("#7B1FA2", "#F3E5F5"),       // Purple
                "DATASTORES" => ("#1976D2", "#E3F2FD"),      // Blue
                "NETWORK" => ("#388E3C", "#E8F5E9"),         // Green
                "SECURITY" => ("#C2185B", "#FFEBEE"),        // Pink
                "CORE" | "SYSTEM" => ("#455A64", "#ECEFF1"), // Blue Gray
                "INFRASTRUCTURE" => ("#5D4037", "#EFEBE9"),  // Brown
                "PRODUCTIVITY" => ("#E64A19", "#FFF3E0"),    // Orange
                "ENTERTAINMENT" => ("#FBC02D", "#FFFDE7"),   // Amber
                _ => ("#616161", "#F5F5F5"),                 // Gray
            };

            let stats_label = if fast_sum > 0 && bulk_sum > 0 {
                format!("{fast_sum}Gi Fast, {bulk_sum}Gi Bulk")
            } else if fast_sum > 0 {
                format!("{fast_sum}Gi Fast")
            } else {
                format!("{bulk_sum}Gi Bulk")
            };

            dot.push_str(&format!("    subgraph cluster_{safe_domain} {{\n"));
            dot.push_str(&format!(
                "      label = \"Domain: {domain_upper} ({stats_label})\";\n"
            ));
            dot.push_str("      style = filled;\n");
            dot.push_str(&format!("      color = \"{header_color}\";\n"));
            dot.push_str(&format!("      fillcolor = \"{bg_color}\";\n"));

            for (name, _dom, storage_gib, (target, base_path), _type) in apps {
                let safe_name = name.replace("-", "_");
                let app_id = format!("app_{safe_name}");

                // Proportional edge width (scale 5Gi->1, 50Gi->4)
                let penwidth = if storage_gib > 0 {
                    1.0 + (storage_gib as f32 / 10.0).min(4.0)
                } else {
                    1.0
                };

                // HTML-like label for structured app info
                let sub_path = base_path.as_deref().unwrap_or("Dynamic");
                let label = format!(
                    "<<TABLE BORDER=\"0\" CELLBORDER=\"0\" CELLSPACING=\"0\">\
                     <TR><TD BGCOLOR=\"{header_color}\" ALIGN=\"LEFT\"><FONT COLOR=\"white\"><B>  {name}  </B></FONT></TD></TR>\
                     <TR><TD ALIGN=\"LEFT\" PORT=\"path\"><FONT POINT-SIZE=\"10\">Path: {sub_path}/{name}</FONT></TD></TR>\
                     <TR><TD ALIGN=\"LEFT\"><FONT POINT-SIZE=\"10\">Size: {storage_gib}Gi</FONT></TD></TR>\
                     </TABLE>>"
                );

                dot.push_str(&format!(
                    "      {app_id} [label={label}, shape=plaintext, style=\"filled\", fillcolor=\"#ffffff:#f5f5f5\", gradientangle=90, tooltip=\"App: {name} - Domain: {domain}\"];\n"
                ));

                dot.push_str(&format!(
                    "      {app_id} -> {target} [label=\"{storage_gib}Gi\", penwidth={penwidth:.1}, color=\"gray40\"];\n"
                ));
            }
            dot.push_str("    }\n");
        }
        dot.push_str("  }\n"); // End cluster_apps

        // ---------------------------------------------------------
        // 2c. Link Paths -> Devices
        // ---------------------------------------------------------
        for (path_id, path_val) in path_nodes {
            let mut matched = false;

            for dev in &devices {
                // Logic: if device mountpoint is prefix of path
                if let Some(mnt) = &dev.mountpoint {
                    if path_val.starts_with(mnt) {
                        let safe_dev = dev.name.replace("-", "_");
                        dot.push_str(&format!(
                            "    {path_id} -> dev_{safe_dev} [style=bold, color=\"#ef6c00\"];\n"
                        ));
                        matched = true;
                        break;
                    }
                }
                // Heuristic Fallback: path name contains device name
                if !matched && path_val.contains(&dev.name) {
                    let safe_dev = dev.name.replace("-", "_");
                    dot.push_str(&format!(
                        "    {path_id} -> dev_{safe_dev} [style=dotted, label=\"implied\"];\n"
                    ));
                    matched = true;
                    break;
                }
            }
            let _ = matched; // Diagnostic: intentionally consumed if nothing else
        }

        // Add virtual default node if needed
        if dot.contains("virtual_default") {
            dot.push_str(
                "  virtual_default [label=\"Default Class\", shape=ellipse, style=dotted];\n",
            );
        }
    }

    // 3. Render Legend
    dot.push_str("  subgraph cluster_legend {\n");
    dot.push_str("    label = \"Legend & Metadata\";\n");
    dot.push_str("    style = filled;\n");
    dot.push_str("    color = \"#eeeeee\";\n");
    dot.push_str("    rank = \"sink\";\n"); // Keep legend at the bottom

    dot.push_str("    legend_node [shape=plaintext, label=<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\">\
        <TR><TD COLSPAN=\"2\" BGCOLOR=\"#bdbdbd\"><B>Storage Tier Legend</B></TD></TR>\
        <TR><TD BGCOLOR=\"#E3F2FD\"><FONT COLOR=\"#1976D2\">Datastores</FONT></TD><TD>Blue Cluster</TD></TR>\
        <TR><TD BGCOLOR=\"#E8F5E9\"><FONT COLOR=\"#388E3C\">Network</FONT></TD><TD>Green Cluster</TD></TR>\
        <TR><TD BGCOLOR=\"#FFEBEE\"><FONT COLOR=\"#C2185B\">Security</FONT></TD><TD>Pink Cluster</TD></TR>\
        <TR><TD BGCOLOR=\"#F3E5F5\"><FONT COLOR=\"#7B1FA2\">Analytics</FONT></TD><TD>Purple Cluster</TD></TR>\
        <TR><TD BGCOLOR=\"#0288d1\"><FONT COLOR=\"white\">Physical</FONT></TD><TD>NVMe/HDD</TD></TR>\
        <TR><TD BGCOLOR=\"#ef6c00\"><FONT COLOR=\"white\">PVC Link</FONT></TD><TD>Provisioned Path</TD></TR>\
        </TABLE>>];\n");
    dot.push_str("  }\n");

    dot.push_str("}\n");

    // Parse OutputFormat
    let output_format = match format.to_lowercase().as_str() {
        "svg" => OutputFormat::Svg,
        "dot" => OutputFormat::Dot,
        "png" => OutputFormat::Png,
        other => anyhow::bail!("Unknown format: {other}"),
    };

    // Determine output path default if needed
    let final_output = output.or_else(|| {
        Some(format!(
            "data/output/diagrams/system-storage.{}",
            format.to_lowercase()
        ))
    });

    if let Some(output_path) = final_output.as_deref() {
        if let Some(parent) = std::path::Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let visualizer =
        bootstrappo::application::runtime::modules::support::visualizer::VisualizerAdapter::new();
    visualizer
        .render(
            &dot,
            output_format,
            &layout,
            final_output.as_deref().map(std::path::Path::new),
        )
        .await
        .context("Failed to render storage visualization")?;

    Ok(())
}
