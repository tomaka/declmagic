extern crate std;
extern crate sync;

pub struct CommandsThread {
	sender: Sender<Message>
}

enum Message {
	Stop(Sender<()>),
	Execute(proc():Send)
}

impl CommandsThread {
	pub fn new() -> CommandsThread {
		let (tx, rx) : (Sender<Message>, Receiver<Message>) = std::comm::channel();

		spawn(proc() {
			loop {
				match rx.recv() {
					Stop(s) => { s.send(()); return; },
					Execute(f) => f()
				}
			}
		});

		CommandsThread {
			sender: tx
		}
	}

	pub fn exec<T:Send>(&self, exec: proc():Send -> T) -> std::sync::Future<T> {
		let (tx, rx) : (Sender<T>, Receiver<T>) = std::comm::channel();
		self.sender.send(Execute(proc() {
			match tx.send_opt(exec()) {
				_ => ()
			}
		}));
		return std::sync::Future::from_receiver(rx);
	}
}

impl Drop for CommandsThread {
	fn drop(&mut self) {
		let (tx, rx) = std::comm::channel();
		self.sender.send(Stop(tx));
		rx.recv();
	}
}
