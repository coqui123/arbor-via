use crate::{
    repo::{
        frogol_repo::FrogolRepo, lead_repo::LeadRepo, link_repo::LinkRepo, user_repo::UserRepo,
    },
    services::{frogol_service::FrogolService, lead_service::LeadService, auth_service::AuthService},
};
use sqlx::SqlitePool;
use std::sync::Arc;


pub struct Services {
    pub frogol: Arc<FrogolService>,
    pub lead: Arc<LeadService>,
    pub auth: Arc<AuthService>,

}

#[derive(Clone)]
pub struct AppState {
    pub services: Arc<Services>,
}

impl AppState {
    pub fn new(pool: SqlitePool, jwt_secret: String) -> Self {
        // Initialize repositories
        let frogol_repo = Arc::new(FrogolRepo::new(pool.clone()));
        let lead_repo = Arc::new(LeadRepo::new(pool.clone()));
        let link_repo = Arc::new(LinkRepo::new(pool.clone()));
        let user_repo = UserRepo::new(pool.clone());



        // Initialize services
        let services = Arc::new(Services {
            frogol: Arc::new(FrogolService::new(frogol_repo, link_repo)),
            lead: Arc::new(LeadService::new(lead_repo)),
            auth: Arc::new(AuthService::new(user_repo, jwt_secret)),
        });

        Self {
            services,
        }
    }
}
