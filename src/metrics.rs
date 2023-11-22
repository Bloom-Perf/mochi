use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;

#[derive(Clone)]
pub struct MochiMetrics {
    mochi_route_not_found_counter: opentelemetry::metrics::Counter<u64>,
}

impl MochiMetrics {
    pub fn create() -> MochiMetrics {
        let meter_provider = opentelemetry::global::meter_provider();

        let my_meter = meter_provider.meter("mochi");

        MochiMetrics {
            mochi_route_not_found_counter: my_meter.u64_counter("mochi_route_not_found").init(),
        }
    }

    pub fn mochi_route_not_found(&self, system: String) {
        self.mochi_route_not_found_counter
            .add(1, &[KeyValue::new("system", system)])
    }
}
