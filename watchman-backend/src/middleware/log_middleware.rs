use actix_service::forward_ready;
use actix_web::body::EitherBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web::Data;
use actix_web::Error;
use actix_web::HttpResponse;
use futures::future::{ok, LocalBoxFuture, Ready};
use log::{error, info};

use share_lib::logger::ShareLogger;

pub struct Logger;