use std::sync::Arc;

use mavlink::ardupilotmega;

use crate::connection::{FilterRes, MavlinkConnection, MavlinkConnectionError};

#[async_trait::async_trait]
pub trait ChangeMode {
    async fn change_mode<C>(self, connection: Arc<C>) -> Result<(), MavlinkConnectionError>
    where
        C: MavlinkConnection + Send + Sync;
}

#[async_trait::async_trait]
impl ChangeMode for ardupilotmega::PlaneMode {
    async fn change_mode<C>(self, connection: Arc<C>) -> Result<(), MavlinkConnectionError>
    where
        C: MavlinkConnection + Send + Sync,
    {
        connection
            .clone()
            .send_wait(
                &ardupilotmega::MavMessage::COMMAND_LONG(ardupilotmega::COMMAND_LONG_DATA {
                    target_system: connection.target_system(),
                    target_component: connection.target_component(),
                    command: ardupilotmega::MavCmd::MAV_CMD_DO_SET_MODE,
                    param1: ardupilotmega::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED.bits()
                        as f32,
                    param2: self as i32 as f32,
                    ..Default::default()
                }),
                std::time::Duration::from_secs(1),
                move |msg| {
                    if let ardupilotmega::MavMessage::HEARTBEAT(beat) = msg {
                        if beat.custom_mode == self as i32 as u32 {
                            return FilterRes::Ready(Some(()));
                        }
                    }
                    FilterRes::NotReady
                },
            )
            .await
            .map(|_| ())
    }
}
