#[derive(Clone, Copy)]
pub(super) enum DepCategory {
    Database,
    Storage,
    Security,
    Network,
    Infrastructure,
    Other,
}

pub(super) fn classify_dependency(dep: &str) -> DepCategory {
    let d = dep.to_lowercase();
    if d.contains("postgres")
        || d.contains("redis")
        || d.contains("mongo")
        || d.contains("qdrant")
        || d.contains("sql")
        || d.contains("db")
        || d.contains("data")
    {
        return DepCategory::Database;
    }
    if d.contains("minio") || d.contains("longhorn") || d.contains("s3") || d.contains("storage") {
        return DepCategory::Storage;
    }
    if d.contains("oidc")
        || d.contains("authelia")
        || d.contains("secret")
        || d.contains("cert")
        || d.contains("vault")
        || d.contains("auth")
    {
        return DepCategory::Security;
    }
    if d.contains("ingress") || d.contains("dns") || d.contains("network") || d.contains("proxy") {
        return DepCategory::Network;
    }
    if d.contains("kro") || d.contains("cnpg") || d.contains("operator") {
        return DepCategory::Infrastructure;
    }
    DepCategory::Other
}
