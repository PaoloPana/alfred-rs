use std::future::Future;
use crate::config::Config;
use crate::pubsub_connection::PubSubConnection;
use crate::connections::connection::{Receiver, Sender};
use crate::error::Error;
use crate::message::Message;

pub struct InterfaceModule {
    pub module_name: String,
    pub config: Config,
    pub connection: PubSubConnection
}

impl InterfaceModule {
    pub async fn new(module_name: String) -> Result<InterfaceModule, Error> {
        let config = Config::read(Some(module_name.clone()))?;
        let connection = PubSubConnection::new(&config).await?;
        Ok(InterfaceModule { module_name, config, connection })
    }
}

impl Receiver for InterfaceModule {
    fn listen(&mut self, topic: String) -> impl Future<Output=Result<(), Error>> {
        self.connection.subscriber.listen(topic)
    }

    async fn receive(&mut self) -> Result<(String, Message), Error> {
        let mut received = false;
        let mut topic= "".to_string();
        let mut message: Message = Message::empty();
        while !received {
            (topic, message) = self.connection.subscriber.receive().await?;
            received = !self.connection.manage_module_info_request(topic.clone(), self.module_name.clone()).await?;
        }
        Ok((topic, message))
    }
}

impl Sender for InterfaceModule {
    fn send(&mut self, topic: String, message: &Message) -> impl Future<Output=Result<(), Error>> {
        self.connection.publisher.send(topic, message)
    }
}