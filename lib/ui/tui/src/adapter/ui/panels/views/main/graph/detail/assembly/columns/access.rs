use primer::application::readiness::ResourceStatus;

use crate::app::App;
use phenome_domain::AssemblyStep;

pub(super) fn gather_ingress_urls(app: &App, step: &AssemblyStep) -> Vec<String> {
    let mut ingress_urls = Vec::new();
    let all_urls = app.context.ports.bootstrap.access_urls();

    for info in &all_urls {
        let svc_lower = info.service.to_lowercase();
        let id_lower = step.id.to_lowercase();
        let svc_norm = svc_lower.replace('-', "");
        let id_norm = id_lower.replace('-', "");

        if svc_lower.contains(&id_lower)
            || id_lower.contains(&svc_lower)
            || svc_norm.contains(&id_norm)
            || id_norm.contains(&svc_norm)
        {
            ingress_urls.push(info.url.clone());
        }
    }

    ingress_urls
}

pub(super) fn gather_ip_info(app: &App, step: &AssemblyStep) -> Option<String> {
    let Ok(details) = app.context.ports.bootstrap.get_detailed_status(&step.id) else {
        return None;
    };
    if let ResourceStatus::Service {
        cluster_ip,
        load_balancer_ip,
    } = details.resource_status
    {
        if let Some(lb) = load_balancer_ip {
            return Some(format!("LB IP: {lb}"));
        }
        if let Some(cip) = cluster_ip {
            return Some(format!("ClusterIP: {cip}"));
        }
    }
    None
}
