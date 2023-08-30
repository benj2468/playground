use std::{fmt::Debug, sync::Arc};

use crate::connection::{MavlinkConnection, MavlinkConnectionError};

#[macro_use]
mod macros;

pub trait Command<Msg, Ack>
where
    Msg: Send + Sync
{
    fn command<C>(&self, connection: Arc<C>) -> Result<usize, MavlinkConnectionError>
    where
        C: MavlinkConnection<Msg> + Debug + Send + Sync;

    fn command_monitor<C>(
        &self,
        connection: Arc<C>,
        timeout: Option<std::time::Duration>,
    ) -> tokio::sync::watch::Receiver<Option<Ack>>
    where
        C: MavlinkConnection<Msg> + Debug + Send + Sync;

    fn command_retry<C>(
        &self,
        _connection: Arc<C>,
        _timeout: std::time::Duration,
        _retry_max: u8,
    ) -> Result<Option<Ack>, MavlinkConnectionError>
    where
        C: MavlinkConnection<Msg> + Debug + Send + Sync,
    {
        unimplemented!("Command Retries are not implemented")
    }
}

command_utils!(mavlink::ardupilotmega);

#[cfg(test)]
mod test;
