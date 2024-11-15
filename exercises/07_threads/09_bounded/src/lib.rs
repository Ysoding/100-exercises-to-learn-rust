// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
    capacity: usize,
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, String> {
        let (sender, receiver) = sync_channel(self.capacity);
        self.sender
            .send(Command::Insert {
                draft,
                response_channel: sender,
            })
            .map_err(|_| "Failed to send Insert command".to_string())?;
        receiver
            .recv()
            .map_err(|_| "Failed to receive response for Insert".to_string())
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, String> {
        let (sender, receiver) = sync_channel(self.capacity);
        self.sender
            .send(Command::Get {
                id,
                response_channel: sender,
            })
            .map_err(|_| "Failed to send Get command".to_string())?;
        receiver
            .recv()
            .map_err(|_| "Failed to receive response for Get".to_string())
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender, capacity }
}

pub enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Ticket>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                response_channel.send(id).unwrap();
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                response_channel.send(ticket.cloned()).unwrap();
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
