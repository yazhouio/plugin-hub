use std::sync::Arc;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum HubError {
    #[error("Tar package with hash '{0}' not exist")]
    TarNotExist(String), // 1000

    #[error("File '{0}' not exist")]
    FileNotExist(String), // 1001

    #[error("Directory '{0}' not exist")]
    DirNotExist(String), // 1002

    #[error("Can not found the config")]
    ConfigNotExist, // 1003

    #[error("Configure error: {0}")]
    ConfigureError(String), // 1004

    #[error("Directory '{0}' has exist")]
    DirHasExist(String), // 1005

    #[error(transparent)]
    IOError(#[from] std::io::Error), // 1006

    #[error(transparent)]
    OtherError(#[from] anyhow::Error), // 1007

    // detailed errors
    #[error("Unsupported API: {0}")]
    UnsupportedApi(String), // 1100
    #[error("Malformed API response for {0}")]
    MalformedApiResponse(String), // 1101

    #[error("Unsupported error code")]
    UnSupportedErrorCode, // 1102

    // converted errors
    #[error("Protobuf decode error: {0}")]
    ProstDecodeError(#[from] prost::DecodeError), // 1200
    #[error("Protobuf decode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError), // 1201
}

#[derive(Debug)]
pub enum HubErrorCode {
    TarNotExist = 1000,
    FileNotExist = 1001,
    DirNotExist = 1002,
    ConfigNotExist = 1003,
    ConfigureError = 1004,
    DirHasExist = 1005,
    IOError = 1006,
    OtherError = 1007,
    UnsupportedApi = 1100,
    MalformedApiResponse = 1101,
    UnSupportedErrorCode = 1102,
    ProstDecodeError = 1200,
    ProstEncodeError = 1201,
}

impl From<&HubError> for i32 {
    fn from(err: &HubError) -> Self {
        match err {
            HubError::TarNotExist(_) => 1000_i32,
            HubError::FileNotExist(_) => 1001_i32,
            HubError::DirNotExist(_) => 1002_i32,
            HubError::ConfigNotExist => 1003_i32,
            HubError::ConfigureError(_) => 1004_i32,
            HubError::DirHasExist(_) => 1005_i32,
            HubError::IOError(_) => 1006_i32,
            HubError::OtherError(_) => 1007_i32,
            HubError::UnsupportedApi(_) => 1100_i32,
            HubError::MalformedApiResponse(_) => 1101_i32,
            HubError::UnSupportedErrorCode => 1102_i32,
            HubError::ProstDecodeError(_) => 1200_i32,
            HubError::ProstEncodeError(_) => 1201_i32,
        }
    }
}

impl From<&HubError> for Vec<u8> {
    fn from(value: &HubError) -> Self {
        i32_to_vec_u8(value.into())
    }
}

pub fn i32_to_vec_u8(code: i32) -> Vec<u8> {
    let code = code.to_be_bytes();
    code.to_vec()
}

impl TryFrom<&[u8]> for HubErrorCode {
    type Error = HubError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let bytes: &[u8; 4] = bytes
            .try_into()
            .map_err(|_| HubError::UnSupportedErrorCode)?;
        let code: i32 = i32::from_be_bytes(bytes.to_owned());
        match code {
            1000 => Ok(HubErrorCode::TarNotExist),
            1001 => Ok(HubErrorCode::FileNotExist),
            1002 => Ok(HubErrorCode::DirNotExist),
            1003 => Ok(HubErrorCode::ConfigNotExist),
            1004 => Ok(HubErrorCode::ConfigureError),
            1005 => Ok(HubErrorCode::DirHasExist),
            1006 => Ok(HubErrorCode::IOError),
            1007 => Ok(HubErrorCode::OtherError),
            1100 => Ok(HubErrorCode::UnsupportedApi),
            1101 => Ok(HubErrorCode::MalformedApiResponse),
            1102 => Ok(HubErrorCode::UnSupportedErrorCode),
            1200 => Ok(HubErrorCode::ProstDecodeError),
            1201 => Ok(HubErrorCode::ProstEncodeError),
            _ => Err(HubError::UnSupportedErrorCode),
        }
    }
}

impl From<HubError> for Status {
    fn from(err: HubError) -> Self {
        let bytes: Vec<u8> = (&err).into();
        let error_clone = Arc::new(err);

        let status = match error_clone.as_ref() {
            HubError::TarNotExist(hash) => {
                Status::not_found(format!("Tar package with hash '{}' not exist", hash))
            }
            HubError::FileNotExist(file) => Status::not_found(format!("File '{}' not exist", file)),
            HubError::DirNotExist(dir) => {
                Status::not_found(format!("Directory '{}' not exist", dir))
            }
            HubError::ConfigNotExist => Status::not_found("Can not found the config"),
            HubError::ConfigureError(msg) => {
                Status::invalid_argument(format!("Configure error: {}", msg))
            }
            HubError::IOError(msg) => Status::internal(format!("IO error: {}", msg)),
            HubError::UnsupportedApi(api) => {
                Status::unimplemented(format!("Unsupported API: {}", api))
            }
            HubError::MalformedApiResponse(api) => {
                Status::internal(format!("Malformed API response for {}", api))
            }
            HubError::ProstDecodeError(err) => {
                Status::internal(format!("Protobuf decode error: {}", err))
            }
            HubError::ProstEncodeError(err) => {
                Status::internal(format!("Protobuf encode error: {}", err))
            }
            HubError::OtherError(err) => Status::internal(format!("Other error: {}", err)),
            HubError::DirHasExist(dir) => {
                Status::invalid_argument(format!("Directory '{}' has exist", dir))
            }
            HubError::UnSupportedErrorCode => Status::internal("Unsupported error code"),
        };
        let status = Status::with_details(status.code(), status.message(), bytes.into());
        status
    }
}

// impl From<HubError> for abi::AppError {
//     fn from(value: HubError) -> Self {
//         match value {
//             HubError::ConfigNotExist => abi::AppError {
//                 code: abi::AppErrorCode::ConfigNotExist.into(),
//                 message: Some("Can not found the config".to_string()),
//             },
//             HubError::TarNotExist(hash) => abi::AppError {
//                 code: abi::AppErrorCode::TarNotExist.into(),
//                 message: Some(format!("Tar package with hash '{}' not exist", hash)),
//             },
//             HubError::FileNotExist(file) => abi::AppError {
//                 code: abi::AppErrorCode::FileNotExist.into(),
//                 message: Some(format!("File '{}' not exist", file)),
//             },
//             HubError::DirNotExist(dir) => abi::AppError {
//                 code: abi::AppErrorCode::DirNotExist.into(),
//                 message: Some(format!("Directory '{}' not exist", dir)),
//             },
//             HubError::ConfigureError(msg) => abi::AppError {
//                 code: abi::AppErrorCode::ConfigureError.into(),
//                 message: Some(format!("Configure error: {}", msg)),
//             },
//             HubError::IOError(msg) => abi::AppError {
//                 code: abi::AppErrorCode::IoError.into(),
//                 message: Some(format!("IO error: {}", msg)),
//             },
//             HubError::UnsupportedApi(api) => abi::AppError {
//                 code: abi::AppErrorCode::UnsupportedApi.into(),
//                 message: Some(format!("Unsupported API: {}", api)),
//             },
//             HubError::MalformedApiResponse(api) => abi::AppError {
//                 code: abi::AppErrorCode::MalformedApiResponse.into(),
//                 message: Some(format!("Malformed API response for {}", api)),
//             },
//             HubError::ProstDecodeError(err) => abi::AppError {
//                 code: abi::AppErrorCode::ProstDecodeError.into(),
//                 message: Some(format!("Protobuf decode error: {}", err)),
//             },
//             HubError::ProstEncodeError(err) => abi::AppError {
//                 code: abi::AppErrorCode::ProstEncodeError.into(),
//                 message: Some(format!("Protobuf encode error: {}", err)),
//             },
//             HubError::OtherError(err) => abi::AppError {
//                 code: abi::AppErrorCode::Other.into(),
//                 message: Some(format!("Other error: {}", err)),
//             },
//             HubError::DirHasExist(dir) => abi::AppError {
//                 code: abi::AppErrorCode::DirHasExist.into(),
//                 message: Some(format!("Directory '{}' has exist", dir)),
//             },
//         }
//     }
// }
