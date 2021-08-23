use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::TryRecvError,
        Arc, Mutex,
    },
    time::Duration,
};

use bus::{Bus, BusReader};
use crossbeam::channel::{unbounded, Receiver, Sender};
use quad_net::quad_socket::{
    client::QuadSocket,
    server::{Settings, SocketHandle},
};
use serde::{Deserialize, Serialize};

use crate::game_state::{
    state::GameState,
    update::{update_game, Command, UpdateEvent},
};

#[derive(Default)]
struct NetState {
    local_state: Option<ClientToServer>,
    buffer: Vec<u8>,
    received_state: AtomicBool,
}

#[derive(Serialize, Deserialize, Clone)]
enum NetEvent {
    Tick,
    Command(Command),
    Disconnected,
    RequestState,
    State(Arc<GameState>),
    Hello,
}

#[derive(Clone)]
struct Connection {
    receiver: Receiver<NetEvent>,
    sender: Sender<NetEvent>,
}

struct ClientToServer {
    receiver: Mutex<BusReader<NetEvent>>,
    sender: Sender<NetEvent>,
}

struct ClientToServerTemplate {
    receiver_source: Arc<Mutex<Bus<NetEvent>>>,
    sender: Sender<NetEvent>,
}

struct ServerToClients {
    receiver: Receiver<NetEvent>,
    sender: Arc<Mutex<Bus<NetEvent>>>,
}

pub(crate) struct Server {
    command_buffer: Vec<Command>,
    clients: ServerToClients,
    state_requested: bool,
}

pub(crate) struct LocalClient {
    to_local_server: Sender<NetEvent>,
}

pub(crate) struct RemoteConnection {
    buffer: Vec<u8>,
    socket: QuadSocket,
    requested_state: bool,
    recv_message_buffer: Vec<NetEvent>,
    recv_command_buffer: Vec<Command>,
}

impl LocalClient {
    pub fn send_commands(&mut self, commands: impl Iterator<Item = Command>) {
        for command in commands {
            self.to_local_server.send(NetEvent::Command(command)).ok();
        }
    }
}

impl Server {
    pub fn relay_messages(&mut self) {
        for message in self.clients.receiver.try_iter() {
            match &message {
                NetEvent::Command(command) => self.command_buffer.push(command.clone()),
                NetEvent::RequestState => self.state_requested = true,
                _ => (),
            }
            let mut sender = self.clients.sender.lock().unwrap();
            sender.broadcast(message);
        }
    }

    pub fn tick(&mut self, game_state: &mut GameState, events: &mut Vec<UpdateEvent>) {
        let commands = self.command_buffer.drain(..);
        update_game(commands, game_state, events);

        let mut sender = self.clients.sender.lock().unwrap();
        sender.broadcast(NetEvent::Tick);

        if self.state_requested {
            self.state_requested = false;

            sender.broadcast(NetEvent::Hello);
            sender.broadcast(NetEvent::State(Arc::new(game_state.clone())));
        }
    }
}

pub(crate) fn serve() -> (Server, LocalClient) {
    let (client_sender, client_receiver) = unbounded();

    let bus = Arc::new(Mutex::new(Bus::new(1024)));

    let clients = ServerToClients {
        receiver: client_receiver,
        sender: bus.clone(),
    };

    let local_server = ClientToServerTemplate {
        receiver_source: bus,
        sender: client_sender.clone(),
    };

    let local_client = LocalClient {
        to_local_server: client_sender,
    };

    std::thread::spawn(move || {
        let on_message = move |socket: &mut SocketHandle<'_>, state: &mut NetState, bytes| {
            local_on_message(socket, state, bytes, &local_server);
        };

        quad_net::quad_socket::server::listen(
            "127.0.0.1:3300",
            "127.0.0.1:3080",
            Settings {
                on_message,
                on_timer,
                on_disconnect,
                timer: Some(Duration::from_millis(1000 / 60)),
                _marker: Default::default(),
            },
        );
    });

    let server = Server {
        clients,
        command_buffer: Vec::new(),
        state_requested: false,
    };

    (server, local_client)
}

fn local_on_message(
    _socket: &mut SocketHandle<'_>,
    state: &mut NetState,
    bytes: Vec<u8>,
    local_server: &ClientToServerTemplate,
) {
    if bytes.is_empty() {
        return;
    }

    state.buffer.extend_from_slice(&bytes);

    if state.buffer.len() < 4 {
        return;
    }

    use std::convert::TryInto;
    let four_bytes: [u8; 4] = state.buffer[0..4].try_into().unwrap();

    let message_size = u32::from_be_bytes(four_bytes) as usize;

    if state.buffer.len() < message_size + 4 {
        return;
    }

    let message: Result<NetEvent, _> = bincode::deserialize(&state.buffer[4..4 + message_size]);

    match message {
        Ok(message) => {
            if let None = state.local_state {
                state.local_state = Some(ClientToServer {
                    receiver: Mutex::new(local_server.receiver_source.lock().unwrap().add_rx()),
                    sender: local_server.sender.clone(),
                });
            }

            let local_state = state.local_state.as_ref().unwrap();
            local_state.sender.send(message).ok();
        }
        Err(err) => eprintln!("Message malformed: {}.", err),
    };

    state.buffer.drain(0..message_size + 4);
}

fn on_timer(socket: &mut SocketHandle<'_>, state: &NetState) {
    let local_state = match &state.local_state {
        Some(state) => state,
        None => return,
    };

    loop {
        let mut receiver_guard = local_state.receiver.lock().unwrap();

        let message = match receiver_guard.try_recv() {
            Ok(message) => message,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                socket.disconnect();
                return;
            }
        };

        drop(receiver_guard);

        if matches!(message, NetEvent::State(_)) {
            state.received_state.store(true, Ordering::Release);
        }

        if state.received_state.load(Ordering::Acquire) {
            let message_bytes = bincode::serialize::<NetEvent>(&message)
                .expect("Local state should always be serializable");

            // FIXME: Handle disconnect.
            socket
                .send(&u32::to_be_bytes(message_bytes.len() as u32))
                .unwrap();
            socket.send(&message_bytes).unwrap();
        } else {
            // No point in sending events until the client has the state
        }
    }
}

fn on_disconnect(state: &NetState) {
    if let Some(local_state) = &state.local_state {
        local_state.sender.send(NetEvent::Disconnected).ok();
    }
}

impl RemoteConnection {
    fn send_message(&mut self, message: NetEvent) {
        let message =
            bincode::serialize(&message).expect("Local state should always be serializable");

        // FIXME: Handle disconnect.
        self.socket.send(&u32::to_be_bytes(message.len() as u32)).ok();
        self.socket.send(&message).ok();
    }

    pub fn send_messages(&mut self, commands: impl Iterator<Item = Command>) {
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
        "127.0.0.1:3080"
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
        recv_command_buffer: Vec::new(),
    };

    remote_connection
}
