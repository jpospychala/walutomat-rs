pub mod v1;
pub mod v2;


#[derive(Debug)]
pub enum Error {
    SerdeErr(serde_json::Error),
    ReqwestErr(reqwest::Error),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeErr(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestErr(error)
    }
}