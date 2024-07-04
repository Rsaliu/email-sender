use tonic::{transport::Server, Request, Response, Status};

pub use email_proto::email_sender_server::{EmailSender, EmailSenderServer};
use email_proto::{EmailSendReply, EmailSendRequest};
use lettre::message::Message;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use std::sync::Mutex;
use std::env;
#[derive(Debug, Default)]
pub struct ConcreteEmailSender {}
#[derive(Debug, Default,Clone)]
pub struct SmtpConfig{
    smtp_server:String,
    smtp_username:String,
    smtp_password:String,
    smtp_sender_email:String
}

#[tonic::async_trait]
impl EmailSender for ConcreteEmailSender {
    async fn send_email(
        &self,
        request: tonic::Request<EmailSendRequest>,
    ) -> std::result::Result<tonic::Response<EmailSendReply>, tonic::Status> {
        static mut SMTP_CONFIG:Option<Mutex<SmtpConfig>> = None;
        unsafe {
            if let None = SMTP_CONFIG {
                SMTP_CONFIG = Some(Mutex::new(SmtpConfig::default()));
                if let Some(ref smtp_config) = SMTP_CONFIG {
                    let mut temp = smtp_config.lock().unwrap();
                    let temp_str = env::var("SMTP_SERVER").expect("env variable error");
                    temp.smtp_server =  temp_str;
                    let temp_str = env::var("SMTP_USERNAME").expect("env variable error");
                    temp.smtp_username =  temp_str;
                    let temp_str = env::var("SMTP_PASSWORD").expect("env variable error");
                    temp.smtp_password =  temp_str;
                    let temp_str = env::var("SMTP_SENDER_EMAIL").expect("env variable error");
                    temp.smtp_sender_email = temp_str;
                }
            }
        }
        let mut msmtp_config:SmtpConfig = SmtpConfig::default();
        unsafe {
            if let Some(ref smtpconfig) = SMTP_CONFIG {
                let temp = smtpconfig.lock().unwrap();
                msmtp_config = temp.clone();
                //println!("template: {}", *temp);
            }
        }
        println!("Got a request: {:?}", request);
        let payload = request.into_inner();

        let mail_email_address = payload.email_address;
        //let mail_msg_id = payload.message_id;
        let mail_title = payload.title;
        let mail_message = payload.message;

        let email = Message::builder()
            .from(msmtp_config.smtp_sender_email.parse().unwrap())
            .to(mail_email_address.parse().unwrap())
            .subject(mail_title)
            .body(mail_message.to_string())
            .unwrap();
        let creds = Credentials::new(msmtp_config.smtp_username, msmtp_config.smtp_password);

        // Open a remote connection to the SMTP server
        let mailer = SmtpTransport::relay(&msmtp_config.smtp_server)
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        tokio::spawn(async move {
            match mailer.send(&email) {
                Ok(_) => println!("Email sent successfully!"),
                Err(e) => panic!("Could not send email: {:?}", e),
            }
        });

        Ok(Response::new(EmailSendReply {
            message_id: String::from("id"),
            message_status: email_proto::Status::Success as i32,
        }))
    }
}
pub mod email_proto {
    tonic::include_proto!("emailsender"); // The string specified here must match the proto package name
}
