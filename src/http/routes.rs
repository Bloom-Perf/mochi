use crate::core::{ConfCore, SystemCore};
use axum::body::Body;
use axum::http::Request;

use crate::http::handler404;
use crate::MochiRouterState;
use axum::extract::State;
use axum::Router;

impl SystemCore {}

impl ConfCore {
    pub fn build_router(
        &self,
        initial_router: Router<MochiRouterState>,
    ) -> Router<MochiRouterState> {
        let mut global_router: Router<MochiRouterState> = initial_router;

        for system in self.systems.iter() {
            let static_router = system.create_static_router();
            let proxy_router = system.create_proxy_router();

            // Proxy setup

            global_router = global_router
                .nest(&format!("/static/{}", &system.name), static_router)
                .nest(&format!("/proxy/{}", &system.name), proxy_router)
        }

        global_router.fallback(move |m: State<MochiRouterState>, r: Request<Body>| {
            handler404(m, r, "Mochi System".to_string())
        })
    }
}
