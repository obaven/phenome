use serde::Serialize;

use bootstrappo::application::api::BootstrappoApi;
use bootstrappo::domain::models::module::EngineMeta;

#[derive(Serialize)]
struct Catalog {
    modules: Vec<CatalogModule>,
}

#[derive(Serialize)]
struct CatalogModule {
    name: String,
    domain: String,
    kind: String,
    engine: CatalogEngine,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    requires: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    provides: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    checks: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum CatalogEngine {
    Native,
    Helm {
        repo: String,
        chart: String,
        version: String,
        namespace: String,
        release: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        values_template: Option<String>,
    },
    Kro {
        rgd_template: String,
        instance_template: String,
    },
    Terraform {
        template_path: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        secrets: Vec<String>,
    },
}

pub async fn catalog(output: Option<String>) -> anyhow::Result<()> {
    let api = BootstrappoApi::new();
    let mut specs = api.list_modules();
    specs.sort_by(|a, b| a.name.cmp(b.name));

    let modules = specs
        .into_iter()
        .map(|spec| {
            let mut requires = spec.required.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let mut provides = spec.provides.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let mut checks = spec.checks.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            requires.sort();
            provides.sort();
            checks.sort();

            CatalogModule {
                name: spec.name.to_string(),
                domain: spec.domain.to_string(),
                kind: spec.kind.to_string().to_lowercase(),
                engine: map_engine(spec.engine),
                requires,
                provides,
                checks,
            }
        })
        .collect::<Vec<_>>();

    let catalog = Catalog { modules };
    let yaml = serde_yaml::to_string(&catalog)?;

    if let Some(path) = output {
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, yaml)?;
        println!("Catalog written to {}", path);
    } else {
        print!("{}", yaml);
    }

    Ok(())
}

fn map_engine(engine: Option<EngineMeta>) -> CatalogEngine {
    match engine.unwrap_or(EngineMeta::Native) {
        EngineMeta::Native => CatalogEngine::Native,
        EngineMeta::Helm(meta) => CatalogEngine::Helm {
            repo: meta.repo.to_string(),
            chart: meta.chart.to_string(),
            version: meta.version.to_string(),
            namespace: meta.namespace.to_string(),
            release: meta.release.to_string(),
            values_template: meta.values_template.map(|v| v.to_string()),
        },
        EngineMeta::Kro(meta) => CatalogEngine::Kro {
            rgd_template: meta.rgd_template.to_string(),
            instance_template: meta.instance_template.to_string(),
        },
        EngineMeta::Terraform(meta) => CatalogEngine::Terraform {
            template_path: meta.template_path.to_string(),
            secrets: meta.secrets.iter().map(|s| s.to_string()).collect(),
        },
    }
}
