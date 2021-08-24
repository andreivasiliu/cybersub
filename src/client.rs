use std::sync::Arc;

use quad_net::quad_socket::client::QuadSocket;
use serde::{Serialize, Deserialize};

use crate::game_state::{state::GameState, update::{Command, UpdateEvent}};

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
    fn send_message(&mut self, message: NetEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            if !self.socket.is_wasm_websocket_connected() {
                self.send_message_buffer.push(message);
                return;
            }
        }

        let message =
            bincode::serialize(&message).expect("Local state should always be serializable");

        // FIXME: Handle disconnect.
        self.socket.send(&u32::to_be_bytes(message.len() as u32)).ok();
        self.socket.send(&message).ok();
    }

    pub fn send_messages(&mut self, commands: impl Iterator<Item = Command>) {
        #[cfg(target_arch = "wasm32")]
        if !self.send_message_buffer.is_empty() {
            let mut messages = Vec::new();
            std::mem::swap(&mut self.send_message_buffer, &mut messages);
            for message in messages {
                self.send_message(message);
            }
        }        
        
        if !self.requested_state {
            self.requested_state = true;
            self.send_message(NetEvent::RequestState);
        }

        for command in commands {
            self.send_message(NetEvent::Command(command));
        }
    }

    pub fn receive_messages(&mut self) {
        'recv: while let Some(bytes) = self.socket.try_recv() {
            self.buffer.extend_from_slice(&bytes);

            loop {
                if self.buffer.len() < 4 {
                    continue 'recv;
                }

                use std::convert::TryInto;
                let four_bytes: [u8; 4] = self.buffer[0..4].try_into().unwrap();

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
                    NetEvent::State(new_state) => match Arc::try_unwrap(new_state) {
                        Ok(new_state) => {
                            *state = new_state;
                            events.push(UpdateEvent::GameStateReset);
                        }
                        Err(_new_state) => {
                            unreachable!("Arc should not be cloned; this is the only client");
                        }
                    },
                    NetEvent::Tick => return Some(self.recv_command_buffer.drain(..)),
                    NetEvent::Hello => (),
                }
            }
        }

        None
    }
}

pub(crate) fn connect() -> RemoteConnection {
    let address = if cfg!(target_arch = "wasm32") {
        "ws://127.0.0.1:3080"
    } else {
        "127.0.0.1:3300"
    };

    // FIXME: Make this a string error
    let socket = QuadSocket::connect(address).expect("Failed to connect");

    let remote_connection = RemoteConnection {
        socket,
        buffer: Vec::new(),
        requested_state: false,
        recv_message_buffer: Vec::new(),
        #[cfg(target_arch = "wasm32")]
        send_message_buffer: Vec::new(),
        recv_command_buffer: Vec::new(),
    };

    remote_connection
}
