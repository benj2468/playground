macro_rules! command_utils {
    ($($dialect:ident)::+) => {
        impl $crate::command::Command<$($dialect)::*::MavMessage, $($dialect)::*::COMMAND_ACK_DATA> for $($dialect)::*::COMMAND_INT_DATA {
            fn command<C>(&self, connection: std::sync::Arc<C>) -> Result<usize, $crate::connection::MavlinkConnectionError>
            where
                C: $crate::connection::MavlinkConnection<$($dialect)::*::MavMessage> + std::fmt::Debug + Send + Sync,
            {
                connection.send(&$($dialect)::*::MavMessage::COMMAND_INT(self.clone()))
            }

            fn command_monitor<C>(
                &self,
                connection: std::sync::Arc<C>,
                timeout: Option<std::time::Duration>,
            ) -> tokio::sync::watch::Receiver<Option<$($dialect)::*::COMMAND_ACK_DATA>>
            where
                C: $crate::connection::MavlinkConnection<$($dialect)::*::MavMessage> + Debug + Send + Sync,
            {
                let (tx, rx) = tokio::sync::watch::channel(None);

                connection.clone().monitor(timeout, move |msg| {
                    if let $($dialect)::*::MavMessage::COMMAND_ACK(data) = msg {
                        tx.send(Some(data.clone())).unwrap();
                        if !matches!(data.result, $($dialect)::*::MavResult::MAV_RESULT_IN_PROGRESS) {
                            return None;
                        }
                    }
                    Some(())
                });

                connection
                    .send(&$($dialect)::*::MavMessage::COMMAND_INT(self.clone()))
                    .expect("Could not send message across mavlink connection -- must have closed");

                rx
            }
        }

        impl $crate::command::Command<$($dialect)::*::MavMessage, $($dialect)::*::COMMAND_ACK_DATA> for $($dialect)::*::COMMAND_LONG_DATA {
            fn command<C>(&self, connection: std::sync::Arc<C>) -> Result<usize, $crate::connection::MavlinkConnectionError>
            where
                C: $crate::connection::MavlinkConnection<$($dialect)::*::MavMessage> + std::fmt::Debug + Send + Sync,
            {
                connection.send(&$($dialect)::*::MavMessage::COMMAND_LONG(self.clone()))
            }

            fn command_monitor<C>(
                &self,
                connection: std::sync::Arc<C>,
                timeout: Option<std::time::Duration>,
            ) -> tokio::sync::watch::Receiver<Option<$($dialect)::*::COMMAND_ACK_DATA>>
            where
                C: $crate::connection::MavlinkConnection<$($dialect)::*::MavMessage> + Debug + Send + Sync,
            {
                let (tx, rx) = tokio::sync::watch::channel(None);

                connection.clone().monitor(timeout, move |msg| {
                    if let $($dialect)::*::MavMessage::COMMAND_ACK(data) = msg {
                        tx.send(Some(data.clone())).unwrap();
                        if !matches!(data.result, $($dialect)::*::MavResult::MAV_RESULT_IN_PROGRESS) {
                            return None;
                        }
                    }
                    Some(())
                });

                connection
                    .send(&$($dialect)::*::MavMessage::COMMAND_LONG(self.clone()))
                    .expect("Could not send message across mavlink connection -- must have closed");

                rx
            }
        }
    };
}
