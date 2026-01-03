use std::path::Path;

use crate::ports::PlanPort;

#[derive(Clone)]
pub struct BootstrappoPlanPort {
    plan: Option<bootstrappo::ops::reconciler::plan::Plan>,
    plan_error: Option<String>,
}

impl BootstrappoPlanPort {
    pub fn load(plan_path: &Path) -> Self {
        let (plan, plan_error) =
            match bootstrappo::ops::reconciler::plan::Plan::load(plan_path) {
                Ok(plan) => (Some(plan), None),
                Err(err) => (None, Some(err.to_string())),
            };
        Self { plan, plan_error }
    }

    pub fn plan(&self) -> Option<bootstrappo::ops::reconciler::plan::Plan> {
        self.plan.clone()
    }

    pub fn plan_error(&self) -> Option<String> {
        self.plan_error.clone()
    }
}

impl PlanPort for BootstrappoPlanPort {
    fn plan(&self) -> Option<bootstrappo::ops::reconciler::plan::Plan> {
        self.plan.clone()
    }

    fn plan_error(&self) -> Option<String> {
        self.plan_error.clone()
    }
}
