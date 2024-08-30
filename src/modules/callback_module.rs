use std::future::Future;
use crate::config::Config;
use crate::pubsub_connection::{PubSubConnection, REQUEST_TOPIC};
use crate::connections::connection::{Receiver, Sender};
use crate::error::Error;
use crate::message::Message;
use crate::modules::module::Module;

pub struct CallbackModule {
    pub module_name: String,
    pub config: Config,
    pub connection: PubSubConnection
}

impl CallbackModule {
    pub async fn new(module_name: String) -> Result<Self, Error> {
        let config = Config::read(Some(module_name.clone()))?;
        let mut connection = PubSubConnection::new(&config).await?;
        connection.listen(REQUEST_TOPIC.to_string()).await?;
        Ok(Self { module_name, config, connection })
    }

    pub fn is_request_message_for_module(&self, topic: &str, message: &Message) -> bool {
        topic != REQUEST_TOPIC || message.request_topic == self.module_name
    }
}

impl Module for CallbackModule {}

impl Receiver for CallbackModule {
    fn listen(&mut self, topic: String) -> impl Future<Output=Result<(), Error>> {
        self.connection.subscriber.listen(topic)
    }

    async fn receive(&mut self) -> Result<(String, Message), Error> {
        let mut received = false;
        let mut topic= "".to_string();
        let mut message: Message = Message::empty();
        while !received {
            (topic, message) = self.connection.subscriber.receive().await?;
            received = !self.connection.manage_module_info_request(topic.as_str(), self.module_name.clone()).await?;
            if received && !self.is_request_message_for_module(topic.as_str(), &message) {
                received = false;
            }
        }
        Ok((topic, message))
    }
}

impl Sender for CallbackModule {
    fn send(&mut self, topic: String, message: &Message) -> impl Future<Output=Result<(), Error>> {
        self.connection.publisher.send(topic, message)
    }
}