use communication::protocol::{
    ProtocolCommand, ProtocolCommandSender, ProtocolEvent, ProtocolEventReceiver,
};
use models::{Block, BlockHeader, BlockId, SerializationContext};
use time::UTime;
use tokio::{sync::mpsc, time::sleep};

const CHANNEL_SIZE: usize = 256;

pub struct MockProtocolController {
    serialization_context: SerializationContext,
    protocol_command_rx: mpsc::Receiver<ProtocolCommand>,
    protocol_event_tx: mpsc::Sender<ProtocolEvent>,
}

impl MockProtocolController {
    pub fn new(
        serialization_context: SerializationContext,
    ) -> (Self, ProtocolCommandSender, ProtocolEventReceiver) {
        let (protocol_command_tx, protocol_command_rx) =
            mpsc::channel::<ProtocolCommand>(CHANNEL_SIZE);
        let (protocol_event_tx, protocol_event_rx) = mpsc::channel::<ProtocolEvent>(CHANNEL_SIZE);
        (
            MockProtocolController {
                serialization_context,
                protocol_event_tx,
                protocol_command_rx,
            },
            ProtocolCommandSender(protocol_command_tx),
            ProtocolEventReceiver(protocol_event_rx),
        )
    }

    pub async fn wait_command<F, T>(&mut self, timeout: UTime, filter_map: F) -> Option<T>
    where
        F: Fn(ProtocolCommand) -> Option<T>,
    {
        let timer = sleep(timeout.into());
        tokio::pin!(timer);
        loop {
            tokio::select! {
                cmd_opt = self.protocol_command_rx.recv() => match cmd_opt {
                    Some(orig_cmd) => if let Some(res_cmd) = filter_map(orig_cmd) { return Some(res_cmd); },
                    None => panic!("Unexpected closure of protocol command channel."),
                },
                _ = &mut timer => return None
            }
        }
    }

    // Note: if you care about the operation set, use another method.
    pub async fn receive_block(&mut self, block: Block) {
        let block_id = block
            .header
            .compute_block_id(&self.serialization_context)
            .unwrap();
        self.protocol_event_tx
            .send(ProtocolEvent::ReceivedBlock {
                block_id,
                block,
                operation_set: Default::default(),
            })
            .await
            .expect("could not send protocol event");
    }

    pub async fn receive_header(&mut self, header: BlockHeader) {
        let block_id = header
            .compute_block_id(&self.serialization_context)
            .unwrap();
        self.protocol_event_tx
            .send(ProtocolEvent::ReceivedBlockHeader { block_id, header })
            .await
            .expect("could not send protocol event");
    }

    pub async fn receive_get_active_blocks(&mut self, list: Vec<BlockId>) {
        self.protocol_event_tx
            .send(ProtocolEvent::GetBlocks(list))
            .await
            .expect("could not send protocol event");
    }

    // ignore all commands while waiting for a futrue
    pub async fn ignore_commands_while<FutureT: futures::Future + Unpin>(
        &mut self,
        mut future: FutureT,
    ) -> FutureT::Output {
        loop {
            tokio::select!(
                res = &mut future => return res,
                cmd = self.protocol_command_rx.recv() => match cmd {
                    Some(_) => {},
                    None => return future.await,  // if the protocol controlled dies, wait for the future to finish
                }
            );
        }
    }
}
