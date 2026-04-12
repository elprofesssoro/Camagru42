use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::{PgPool};
use std::io::Error;
use std::env;

pub struct AppState {
    pub db: PgPool,
	pub email_conf: EmailConfig
}

#[derive(Clone)]
pub struct EmailConfig {
	email: String,
	password: String,
}

impl EmailConfig {
	pub fn get_email(&self) -> String {
		self.email.clone()
	}
	pub fn get_pass(&self) -> String {
		self.password.clone()
	}
	pub fn get_env() -> Result<Self, Error> {
    	let email = env::var("EMAIL_HOST")
    	    .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
    	let password = env::var("PASSWORD_HOST")
    	    .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
		
    	Ok(Self { email, password })
	}
}

pub async fn send_email(email_conf: EmailConfig, from: String, to: String, subject: String, body: String) {
    let (username, password) = (email_conf.get_email(), email_conf.get_pass());

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .unwrap();

    println!("{}, {}", username, password);
    let creds = Credentials::new(username, password);

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

    match mailer.send(email).await {
		Ok(_) => println!("Email sent successfully!"),
        Err(e) => log_error("Could not send email: {:?}", e),
	}
}

pub fn log_error(context: &str, err: impl std::fmt::Display) {
    eprintln!("[ERROR] {} - {}", context, err);
}
