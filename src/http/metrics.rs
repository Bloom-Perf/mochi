use opentelemetry::metrics::{Counter, Meter, MeterProvider};
use opentelemetry::KeyValue;

#[derive(Clone)]
pub struct MochiMetrics {
    mochi_route_not_found_counter: opentelemetry::metrics::Counter<u64>,
    mochi_proxy_request_counter: opentelemetry::metrics::Counter<u64>,
}

impl MochiMetrics {
    pub fn new() -> MochiMetrics {
        let meter_provider = opentelemetry::global::meter_provider();

        let my_meter = meter_provider.meter("mochi");

        MochiMetrics {
            mochi_route_not_found_counter: my_meter.u64_counter("mochi_route_not_found").init(),
            mochi_proxy_request_counter: my_meter.u64_counter("mochi_proxy_request_counter").init(),
        }
    }

    pub fn mochi_route_not_found(&self, system: String) {
        self.mochi_route_not_found_counter
            .add(1, &[KeyValue::new("system", system)])
    }

    pub fn mochi_proxy_request_counter(
        &self,
        system: &str,
        api: Option<&String>,
        proxy_uri: &str,
        path: &str,
    ) {
        self.mochi_proxy_request_counter.add(
            1,
            &[
                KeyValue::new("system", system.to_owned()),
                KeyValue::new("api", (api.unwrap_or(&"root".to_string())).clone()),
                KeyValue::new("uri", proxy_uri.to_owned()),
                KeyValue::new("path", path.to_owned()),
            ],
        )
    }
}
