use crate::errors::{Error::API, Result};
use aws_sdk_sts::{types::SdkError, Client};
use aws_types::SdkConfig as AwsSdkConfig;
use serde::{Deserialize, Serialize};

/// Implements AWS STS manager.
#[derive(Debug, Clone)]
pub struct Manager {
    #[allow(dead_code)]
    shared_config: AwsSdkConfig,
    cli: Client,
}

impl Manager {
    pub fn new(shared_config: &AwsSdkConfig) -> Self {
        let cloned = shared_config.clone();
        let cli = Client::new(shared_config);
        Self {
            shared_config: cloned,
            cli,
        }
    }

    pub fn client(&self) -> Client {
        self.cli.clone()
    }

    /// Queries the AWS caller identity from the default AWS configuration.
    pub async fn get_identity(&self) -> Result<Identity> {
        log::info!("fetching STS caller identity");
        let ret = self.cli.get_caller_identity().send().await;
        let resp = match ret {
            Ok(v) => v,
            Err(e) => {
                return Err(API {
                    message: format!("failed get_caller_identity {:?}", e),
                    is_retryable: is_error_retryable(&e),
                });
            }
        };

        Ok(Identity::new(
            resp.account().unwrap_or(""),
            resp.arn().unwrap_or(""),
            resp.user_id().unwrap_or(""),
        ))
    }
}

/// Represents the caller identity.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Identity {
    pub account_id: String,
    pub role_arn: String,
    pub user_id: String,
}

impl Identity {
    pub fn new(account_id: &str, role_arn: &str, user_id: &str) -> Self {
        // ref. https://doc.rust-lang.org/1.0.0/style/ownership/constructors.html
        Self {
            account_id: String::from(account_id),
            role_arn: String::from(role_arn),
            user_id: String::from(user_id),
        }
    }
}

#[inline]
pub fn is_error_retryable<E>(e: &SdkError<E>) -> bool {
    match e {
        SdkError::TimeoutError(_) | SdkError::ResponseError { .. } => true,
        SdkError::DispatchFailure(e) => e.is_timeout() || e.is_io(),
        _ => false,
    }
}
