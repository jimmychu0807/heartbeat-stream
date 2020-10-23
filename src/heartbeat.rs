use std::{
	boxed::Box,
	time::Instant,
};

use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
	time::{Duration},
};
use tokio::{
	stream::{Stream},
};
use futures_timer::Delay;

const heartbeat_imported: Instant = Instant::now();

pub struct HeartbeatStream {
	cooldown: Option<Duration>,
	heartbeat: Option<Duration>,
	next_heartbeat: Option<Delay>,
	last_signal: Option<Instant>,
	streams: Vec<Box<dyn Stream<Item = char>>>,
	char: char,
}

impl HeartbeatStream {
	pub fn new(cooldown_sec: Option<u64>, heartbeat_sec: Option<u64>,
		streams: Vec<Box<dyn Stream<Item = char>>>, char: char) -> Self
	{
		Self {
			cooldown: cooldown_sec.map(|s| Duration::from_secs(s)),
			heartbeat: heartbeat_sec.map(|s| Duration::from_secs(s)),
			next_heartbeat: heartbeat_sec.map(|s| Delay::new(Duration::from_secs(s))),
			last_signal: None,
			streams,
			char,
		}
	}

	fn reset_last_signal_next_heartbeat(&mut self) {
		self.last_signal = Some(Instant::now());
		self.next_heartbeat = self.heartbeat.map(|d| Delay::new(d));
	}
}

impl Stream for HeartbeatStream {
	type Item = char;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let mut stream1 = self.streams[0].as_mut();
		let mut stream2 = self.streams[1].as_mut();

		let current = Instant::now();
		let since_last_signal = Instant::now().saturating_duration_since(
			self.last_signal.unwrap_or(heartbeat_imported));

		// We ensure we first poll the `next_heartbeat` delay future
		if let Some(hb) = self.next_heartbeat { Pin::new(&mut hb).poll(cx); }

		tokio::spawn(async {
			tokio::select! {
				char = stream1, if self.cooldown.map_or(true, |cd| since_last_signal > cd) => {
					self.reset_last_signal_next_heartbeat();
					return Poll::Ready(char)
				},
				char = stream2, if self.cooldown.map_or(true, |cd| since_last_signal > cd) => {
					self.reset_last_signal_next_heartbeat();
					return Poll::Ready(char)
				},
				_ = self.next_heartbeat => {
					self.reset_last_signal_next_heartbeat();
					return Poll::Ready(self.char)
				}
			}
		});

		Poll::Pending
	}
}
