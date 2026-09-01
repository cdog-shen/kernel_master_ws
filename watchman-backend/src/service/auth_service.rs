//! 鉴权编排层
//!
//! 抽取 JwtAuth / PermissionCheck 中间件的鉴权判定逻辑，
//! 供中间件与 `/api/auth/verify` 回源鉴权接口共用，保证两处语义一致。

use actix_web::{http::Method, web};
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use serde::Serialize;

use share_lib::data_structure::{MailManErr, MailManOk};

use crate::model::{
    access::AccessModel,
    group::GroupModel,
    service::ServiceModel,
    subsys::SubsysModel,
    user::UserModel,
    user_token::{TokenModel, UserToken},
};

/// 用户身份（token 认证结果）
pub struct AuthIdentity {
    pub uid: i32,
    pub username: String,
}

/// 回源鉴权结果
#[derive(Serialize)]
pub struct VerifyResult {
    pub uid: i32,
    pub username: String,
    /// 2=读写 1=只读 0=无权限
    pub permission: i32,
}

/// 用户 token 认证（与原 JwtAuth 中间件判定逻辑一致）
///
/// 失败语义：
/// - token 解码失败 -> 401
/// - token 表查不到或已过期 -> 401
/// - 用户查询失败（DB 异常等）-> 500
pub async fn authenticate(
    token: &str,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<AuthIdentity, MailManErr<'static, String>> {
    // 解码 JWT
    let claims = match UserToken::decode_token(token.to_string()) {
        Ok(c) => c,
        Err(_) => {
            return Err(MailManErr::new(
                401,
                "Unauthorized",
                Some("invalid or expired token".to_string()),
                1,
            ));
        }
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(format!("db pool error: {e}")),
                1,
            ));
        }
    };

    // token 表校验（存在且未过期）
    let username = match TokenModel::token_ckeck(&claims, &mut conn) {
        Ok(u) => u,
        Err(_) => {
            return Err(MailManErr::new(
                401,
                "Unauthorized",
                Some("token not found in store".to_string()),
                1,
            ));
        }
    };

    // 查用户拿 uid
    let user = match UserModel::get_user_by_username(&username, &mut conn) {
        Ok(u) => u,
        Err(msg) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(msg.1),
                1,
            ));
        }
    };
    let uid = match user.id {
        Some(id) => id,
        None => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some("user id is missing".to_string()),
                1,
            ));
        }
    };

    Ok(AuthIdentity { uid, username })
}

/// 资源权限判定（与原 PermissionCheck 中间件判定逻辑一致）
///
/// 成功返回权限值（2=读写 1=只读）；失败语义：
/// - 权限链 DB 查询失败 -> 500
/// - 权限为 0，或权限为 1 且非 GET 请求 -> 403
pub async fn authorize(
    uid: i32,
    path: &str,
    method: &Method,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<i32, MailManErr<'static, String>> {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(format!("db pool error: {e}")),
                1,
            ));
        }
    };

    let gids = GroupModel::get_groups_by_uid(uid, &mut conn).unwrap_or_default();
    let gid_vec: Vec<i32> = gids.into_iter().map(|g| g.id).collect();
    let sids = ServiceModel::get_sids_by_route(path, &mut conn).unwrap_or_default();

    let perm = match AccessModel::get_max_permission(&gid_vec, &sids, &mut conn) {
        Ok(p) => p,
        Err(e) => {
            return Err(MailManErr::new(500, "Internal Server Error", Some(e.1), 1));
        }
    };

    // 2=读写放行；1=只读仅放行 GET；其余拒绝
    let allowed = match perm {
        2 => true,
        1 => *method == Method::GET,
        _ => false,
    };

    if !allowed {
        return Err(MailManErr::new(
            403,
            "Forbidden",
            Some("User has no permissions on the resource".to_string()),
            1,
        ));
    }

    Ok(perm as i32)
}

/// 调用方子系统认证（供 /api/auth/verify 使用）
///
/// 校验 subsys_name 是否对应已注册且启用的子系统，且 subsys_uuid 与其
/// token 一致；子系统不存在、未启用或 uuid 不符均返回 401
pub async fn authenticate_subsys(
    subsys_name: &str,
    subsys_uuid: &str,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<SubsysModel, MailManErr<'static, String>> {
    let parsed = match uuid::Uuid::parse_str(subsys_uuid) {
        Ok(u) => u,
        Err(_) => {
            return Err(MailManErr::new(
                401,
                "Unauthorized",
                Some("invalid subsystem uuid".to_string()),
                1,
            ));
        }
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            return Err(MailManErr::new(
                500,
                "Internal Server Error",
                Some(format!("db pool error: {e}")),
                1,
            ));
        }
    };

    let subsys = match SubsysModel::get_enable_by_name(&subsys_name.to_string(), &mut conn) {
        Ok(subsys) => subsys,
        Err(msg) => match msg.0 {
            1 => {
                return Err(MailManErr::new(
                    401,
                    "Unauthorized",
                    Some("subsystem not found or disabled".to_string()),
                    1,
                ));
            }
            _ => {
                return Err(MailManErr::new(
                    500,
                    "Internal Server Error",
                    Some(msg.1),
                    1,
                ));
            }
        },
    };

    if subsys.token != parsed {
        return Err(MailManErr::new(
            401,
            "Unauthorized",
            Some("subsystem uuid does not match name".to_string()),
            1,
        ));
    }

    Ok(subsys)
}

/// 回源鉴权：目标资源权限判定 + 结果组装（供 /api/auth/verify 使用）
///
/// 用户 token 认证已由 JwtAuth 中间件完成、子系统 name/uuid 匹配已由
/// handler 完成，此处仅对 body 内目标 path/method 做权限判定
pub async fn verify<'a>(
    uid: i32,
    username: String,
    path: &str,
    method: &Method,
    pool: &web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> Result<MailManOk<'a, VerifyResult>, MailManErr<'a, String>> {
    let permission = authorize(uid, path, method, pool).await?;

    Ok(MailManOk::new(
        200,
        "Service: Auth - verify",
        Some(VerifyResult {
            uid,
            username,
            permission,
        }),
    ))
}
