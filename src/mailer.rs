use derive_more::{Display, Error};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

#[derive(Debug, Display, Error)]
pub enum SendMailError {
    EnvVar(env::VarError),
    Lelay(lettre::transport::smtp::Error),
    Send(lettre::transport::smtp::Error),
}

pub async fn send_mail(message: &Message) -> Result<(), SendMailError> {
    let api_key = env::var("SENDGRID_API_KEY").map_err(SendMailError::EnvVar)?;
    let creds = Credentials::new("apikey".to_string(), api_key);

    let mailer = SmtpTransport::relay("smtp.sendgrid.net")
        .map_err(SendMailError::Lelay)?
        .credentials(creds)
        .build();

    mailer.send(message).map_err(SendMailError::Send)?;

    Ok(())
}
