use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize)]
pub enum StatusCode {
    Ok,
    NoAuthorization,
    InternalError,
    BadRequest,
    NotFound,
}

impl Serialize for StatusCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(match self {
            StatusCode::Ok => 200,
            StatusCode::BadRequest => 400,
            StatusCode::NoAuthorization => 403,
            StatusCode::InternalError => 500,
            StatusCode::NotFound => 404,
        })
    }
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
}
