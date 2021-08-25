use std::sync::Arc;

use quad_net::quad_socket::client::QuadSocket;
use serde::{Deserialize, Serialize};

use crate::game_state::{
    state::GameState,
    update::{Command, UpdateEvent},
};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum NetEvent {
    Tick,
    Command(Command),
    Disconnected,
    RequestState,
    State(Arc<GameState>),
    Hello,
}

pub(crate) struct RemoteConnection {
    buffer: Vec<u8>,
    socket: QuadSocket,
    requested_state: bool,
    recv_message_buffer: Vec<NetEvent>,
    #[cfg(target_arch = "wasm32")]
    send_message_buffer: Vec<NetEvent>,
    recv_command_buffer: Vec<Command>,
}

impl RemoteConnection {
    fn send_message(&mut self, message: NetEvent) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            if !self.socket.is_wasm_websocket_connected() {
                self.send_message_buffer.push(message);
                return Ok(());
            }
        }

        let message =
            bincode::serialize(&message).expect("Local state should always be serializable");

        self.socket
            .send(&u32::to_be_bytes(message.len() as u32))
            .map_err(|err| format!("Could not send message: {:?}", err))?;
        self.socket
            .send(&message)
            .map_err(|err| format!("Could not send message: {:?}", err))?;

        Ok(())
    }

    pub fn send_messages(&mut self, commands: impl Iterator<Item = Command>) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        if !self.send_message_buffer.is_empty() {
            let mut messages = Vec::new();
            std::mem::swap(&mut self.send_message_buffer, &mut messages);
            for message in messages {
                self.send_message(message)?;
            }
        }

        if !self.requested_state {
            self.requested_state = true;
            self.send_message(NetEvent::RequestState)?;
        }

        for command in commands {
            self.send_message(NetEvent::Command(command))?;
        }

        Ok(())
    }

    pub fn receive_messages(&mut self) {
        'recv: while let Some(bytes) = self.socket.try_recv() {
            self.buffer.extend_from_slice(&bytes);

            loop {
                if self.buffer.len() < 4 {
                    continue 'recv;
                }

                use std::convert::TryInto;
                let four_bytes: [u8; 4] = self.buffer[0..4]
                    .try_into()
                    .expect("Sliced exactly 4 bytes");

                let message_size = u32::from_be_bytes(four_bytes) as usize;

                if self.buffer.len() < message_size + 4 {
                    continue 'recv;
                }

                let message: Result<NetEvent, _> =
                    bincode::deserialize(&self.buffer[4..4 + message_size]);

                match message {
                    Ok(message) => self.recv_message_buffer.push(message),
                    Err(err) => eprintln!("Message malformed: {}", err),
                }

                self.buffer.drain(0..message_size + 4);
            }
        }
    }

    pub fn receive_commands(
        &mut self,
        state: &mut GameState,
        events: &mut Vec<UpdateEvent>,
    ) -> Option<impl Iterator<Item = Command> + '_> {
        while let Some(tick_index) = self
            .recv_message_buffer
            .iter()
            .position(|m| matches!(m, NetEvent::Tick))
        {
            for net_event in self.recv_message_buffer.drain(..tick_index + 1) {
                match net_event {
                    NetEvent::Command(command) => self.recv_command_buffer.push(command),
                    NetEvent::Disconnected => (),
                    NetEvent::RequestState => (),
                    NetEvent::State(new_state) => {
                        let new_state = Arc::try_unwrap(new_state)
                            .map_err(|_arc| "Arc was cloned")
                            .expect("Arc should not be cloned; this is the only client");
                        *state = new_state;
                        events.push(UpdateEvent::GameStateReset);
                    }
                    NetEvent::Tick => return Some(self.recv_command_buffer.drain(..)),
                    NetEvent::Hello => (),
                }
            }
        }

        None
    }
}

pub(crate) fn connect(address: &str) -> Result<RemoteConnection, String> {
    // FIXME: Make this a string error
    let socket =
        QuadSocket::connect(address).map_err(|err| format!("Failed to connect: {:?}", err))?;

    let remote_connection = RemoteConnection {
        socket,
        buffer: Vec::new(),
        requested_state: false,
        recv_message_buffer: Vec::new(),
        #[cfg(target_arch = "wasm32")]
        send_message_buffer: Vec::new(),
        recv_command_buffer: Vec::new(),
    };

    Ok(remote_connection)
}
