use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

pub async fn send_mail(message: &Message) -> Result<(), String> {
    let api_key = env::var("SENDGRID_API_KEY").map_err(|e| e.to_string())?;
    let creds = Credentials::new("apikey".to_string(), api_key);

    let mailer = SmtpTransport::relay("smtp.sendgrid.net")
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .build();

    if let Err(e) = mailer.send(message) {
        return Err(e.to_string());
    }
    Ok(())
}
