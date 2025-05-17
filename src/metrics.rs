use prometheus::{Encoder, IntGaugeVec, Opts, Registry, TextEncoder};

pub struct Metrics {
    pub registry: Registry,
    outdated: IntGaugeVec,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        let outdated = IntGaugeVec::new(
            Opts::new("image_outdated", "Container images with outdated tag"),
            &["repository", "current", "latest"],
        )
        .expect("metric can be created");
        registry
            .register(Box::new(outdated.clone()))
            .expect("collector can be registered");
        Self { registry, outdated }
    }

    pub fn set_outdated(&self, outdated_images: &[(String, String, String)]) {
        self.outdated.reset();
        for (repo, current, latest) in outdated_images {
            self.outdated
                .with_label_values(&[repo, current, latest])
                .set(1);
        }
    }

    pub fn gather(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder
            .encode(&metric_families, &mut buffer)
            .expect("encode metrics");
        String::from_utf8(buffer).expect("metrics are valid utf8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_output() {
        let m = Metrics::new();
        m.set_outdated(&[("nginx".into(), "1.0".into(), "1.2".into())]);
        let out = m.gather();
        assert!(out.contains("image_outdated"));
        assert!(out.contains("nginx"));
        assert!(out.contains("1.2"));
    }
}
