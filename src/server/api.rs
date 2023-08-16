use actix_web::{App, HttpServer, middleware, web};
use crate::config::ServerSchema;
use crate::server::{KPGServer, ServerContext};
use crate::server::controller::playlist::{get_playlist_list};
use crate::util::error::KPGError;
use crate::util::error::KPGErrorCode::KPGAPIServerBindFailed;

const MAX_JSON_BODY: usize = 1024 * 1024;

pub struct KPGApi {
    address: String,
    port: u16,
}

impl KPGApi {
    pub fn new(address: String, port: u16) -> KPGApi {
        KPGApi { address, port }
    }

    pub async fn run(&self) -> Result<(), KPGError> {
        HttpServer::new(|| {
            App::new().wrap(middleware::Logger::default())
                .app_data(web::JsonConfig::default().limit(MAX_JSON_BODY))
                .route("/playlist", web::get().to(get_playlist_list))
        }).bind((self.address.as_str(), self.port)).map_err(|err| {
            KPGError::new_with_string(KPGAPIServerBindFailed, format!("address: {}, port: {}, error: {}", self.address, self.port, err))
        })?.run().await.map_err(|err| {
            KPGError::new_with_string(KPGAPIServerBindFailed, format!("address: {}, port: {}, error: {}", self.address, self.port, err))
        })?;

        Ok(())
    }
}

impl KPGServer for KPGApi {
    fn start(&mut self) -> Result<(), KPGError> {
        actix_rt::System::new().block_on(async {
            self.run().await
        })?;

        Ok(())
    }

    fn stop(&mut self) -> Result<(), KPGError> {
        Ok(())
    }

    fn get_schema(&self, schema: ServerSchema) -> Option<ServerContext> {
        todo!()
    }

    fn get_name(&self) -> String {
        todo!()
    }
}