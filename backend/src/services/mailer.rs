use lettre::{
    message::Message,
    transport::smtp::{authentication::Credentials, AsyncSmtpTransport},
    AsyncTransport, Tokio1Executor,
};

use crate::{
    config::SmtpConfig,
    entities::bug_reports,
    error::{AppError, AppResult},
};

/// Sends bug-report notification emails. Built from the SMTP config; `None`
/// when no SMTP host is configured (reports are then stored without mail).
pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
    recipient: String,
}

impl Mailer {
    pub fn from_config(config: &SmtpConfig) -> AppResult<Option<Self>> {
        let Some(host) = config.host.clone() else {
            return Ok(None);
        };
        let from = config.from.clone().ok_or_else(|| {
            AppError::Config("smtp.from is required when smtp.host is set".to_string())
        })?;

        // 465 is the implicit-TLS submission port; anything else (typically
        // 587) goes over STARTTLS.
        let port = config.port.unwrap_or(465);
        let builder = if port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
        }
        .map_err(|e| AppError::Config(format!("smtp tls setup: {}", e)))?
        .port(port);

        let builder = match (config.username.clone(), config.password.clone()) {
            (Some(user), Some(pass)) => builder.credentials(Credentials::new(user, pass)),
            _ => builder,
        };

        Ok(Some(Self {
            transport: builder.build(),
            from,
            recipient: config.report_recipient.clone(),
        }))
    }

    /// Mails a stored bug report to the configured recipient. The reporter's
    /// address (when given) becomes Reply-To so answering the mail reaches
    /// them directly. Attachments are included as presigned download links.
    pub async fn send_bug_report(
        &self,
        report: &bug_reports::Model,
        attachment_links: &[String],
    ) -> AppResult<()> {
        let mut body = format!(
            "New bug report\n\nTitle: {}\nVersion: {}\nEmail: {}\nSubmitted: {}\n\n{}\n",
            report.title,
            report.app_version,
            report.email.as_deref().filter(|e| !e.is_empty()).unwrap_or("(not provided)"),
            report.created_at,
            report.content,
        );
        if !attachment_links.is_empty() {
            body.push_str("\nAttachments (permanent links):\n");
            for link in attachment_links {
                body.push_str(link);
                body.push('\n');
            }
        }

        let builder = Message::builder()
            .from(self.from.parse().map_err(|e| AppError::Internal(format!("invalid smtp from address: {}", e)))?)
            .to(self.recipient.parse().map_err(|e| AppError::Internal(format!("invalid smtp recipient: {}", e)))?)
            .subject(format!("[Oak Bug Report] {}", report.title));

        let builder = match report.email.as_deref().filter(|e| !e.is_empty()) {
            Some(addr) => match addr.parse() {
                Ok(reply_to) => builder.reply_to(reply_to),
                Err(_) => builder,
            },
            None => builder,
        };

        let message = builder
            .body(body)
            .map_err(|e| AppError::Internal(format!("build email: {}", e)))?;

        self.transport
            .send(message)
            .await
            .map_err(|e| AppError::External(format!("send mail: {}", e)))?;
        Ok(())
    }
}
