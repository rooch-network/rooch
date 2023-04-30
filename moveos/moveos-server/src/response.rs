use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusCode {
    Ok = 200,
    NoAuthorization = 400,
    InternalError = 403,
    BadRequest = 500,
    NotFound = 404,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonResponse<T: Serialize> {
    Ok {
        code: StatusCode,
        message: Option<String>,
        data: Option<T>,
    },
    Error {
        code: StatusCode,
        message: Option<String>,
    },
}

impl<T: Serialize> JsonResponse<T> {
    /// ok without any message
    pub fn ok(data: T) -> Self {
        Self::Ok {
            code: StatusCode::Ok,
            message: None,
            data: Some(data),
        }
    }

    /// ok with message
    pub fn ok_with_message(data: T, message: String) -> Self {
        Self::Ok {
            code: StatusCode::Ok,
            message: Some(message),
            data: Some(data),
        }
    }

    /// error without any message
    pub fn error(code: StatusCode) -> Self {
        Self::Error {
            code,
            message: None,
        }
    }

    /// error with message
    pub fn error_with_message(code: StatusCode, message: String) -> Self {
        Self::Error {
            code,
            message: Some(message),
        }
    }

    pub fn try_into_inner(self) -> Result<Option<T>> {
        match self {
            JsonResponse::Ok {
                code,
                message,
                data,
            } => Ok(data),
            JsonResponse::Error { code, message } => {
                bail!("Error response can not convert into type T.")
            }
        }
    }
}
