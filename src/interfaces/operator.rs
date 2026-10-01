// SPEC-MANAGED: tech-design/semantic/pgpool-crd-operator-control-plane.md#logic
// <HANDWRITE gap="missing-generator:logic:4a951ea7" tracker="#1575" reason="Export Pgpool CRD, render, reconcile, CRD YAML normalization, and operator deployment-manifest rendering.">
pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{
    Pgpool, PgpoolEndpointBudgetSpec, PgpoolEndpointBudgetStatus, PgpoolEndpointProvider,
    PgpoolEndpointRole, PgpoolPodBudgetStatus, PgpoolResources, PgpoolSecretKeyRef, PgpoolSpec,
    PgpoolStatus,
};
pub use reconcile::run;
mod manifest;

pub use manifest::{crd_yaml, instance_yaml, operator_manifests, operator_yaml};
