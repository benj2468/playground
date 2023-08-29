use std::{fmt::Debug, sync::Arc};

use mavlink::ardupilotmega::{MavMessage, MavResult, COMMAND_ACK_DATA};

use crate::connection::{MavlinkConnection, MavlinkConnectionError};

pub trait Command {
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync;

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: std::time::Duration,
    ) -> tokio::sync::watch::Receiver<Option<COMMAND_ACK_DATA>>
    where
        C: MavlinkConnection + Debug + Send + Sync;

    fn command_retry<C>(
        &self,
        _connection: Arc<C>,
        _timeout: std::time::Duration,
        _retry_max: u8,
    ) -> Result<Option<COMMAND_ACK_DATA>, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        unimplemented!("Command Retries are not implemented")
    }
}

impl Command for mavlink::ardupilotmega::COMMAND_INT_DATA {
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        connection.send(&MavMessage::COMMAND_INT(self.clone()))
    }

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: std::time::Duration,
    ) -> tokio::sync::watch::Receiver<Option<COMMAND_ACK_DATA>>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        let (tx, rx) = tokio::sync::watch::channel(None);

        connection.clone().monitor(Some(timeout), move |msg| {
            if let MavMessage::COMMAND_ACK(data) = msg {
                if data.result == MavResult::MAV_RESULT_ACCEPTED {
                    return None;
                }
                tx.send(Some(data));
            }
            return Some(());
        });

        connection.send(&MavMessage::COMMAND_INT(self.clone()));

        rx
    }
}

impl Command for mavlink::ardupilotmega::COMMAND_LONG_DATA {
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        connection.send(&MavMessage::COMMAND_LONG(self.clone()))
    }

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: std::time::Duration,
    ) -> tokio::sync::watch::Receiver<Option<COMMAND_ACK_DATA>>
    where
        C: MavlinkConnection + Debug + Send + Sync,
    {
        let (tx, rx) = tokio::sync::watch::channel(None);

        connection.clone().monitor(None, move |msg| {
            if let MavMessage::COMMAND_ACK(data) = msg {
                tx.send(Some(data.clone()));
                if data.result == MavResult::MAV_RESULT_ACCEPTED {
                    return None;
                }
                if !matches!(data.result, MavResult::MAV_RESULT_IN_PROGRESS) {
                    return None;
                }
            }
            return Some(());
        });

        connection.send(&MavMessage::COMMAND_LONG(self.clone()));

        rx
    }
}
