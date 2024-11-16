use actix_web::middleware::{self, Next};
use actix_web::{
    body::MessageBody,
    dev::{Service as _, ServiceRequest, ServiceResponse},
    App, Error,
};

use share_lib::logger::ShareLogger;

// pub struct Logger;

pub async fn logger(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    let res = next.call(req).await?;
    Ok(res)
}
