use derive_more::{Display, Error};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::Error;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use std::env::VarError;

#[derive(Debug, Display, Error)]
pub enum SendMailError {
    EnvVar(VarError),
    Send(lettre::transport::smtp::Error),
}

impl From<VarError> for SendMailError {
    fn from(value: VarError) -> Self {
        Self::EnvVar(value)
    }
}

impl From<lettre::transport::smtp::Error> for SendMailError {
    fn from(value: Error) -> Self {
        Self::Send(value)
    }
}

pub async fn send_mail(message: &Message) -> Result<(), SendMailError> {
    let api_key = env::var("SENDGRID_API_KEY")?;
    let creds = Credentials::new("apikey".to_string(), api_key);

    let mailer = SmtpTransport::relay("smtp.sendgrid.net")?
        .credentials(creds)
        .build();

    mailer.send(message).map_err(SendMailError::Send)?;

    Ok(())
}
