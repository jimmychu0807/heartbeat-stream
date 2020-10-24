use std::{
	boxed::Box,
	marker::{Unpin, Send},
	time::Instant,
};

use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
	time::{Duration},
};
use tokio::{
	stream::{Stream, StreamExt},
};
use futures_timer::Delay;

type VS = Vec<Box<dyn Stream<Item = char> + Unpin + Send>>;
const heartbeat_imported: Instant = Instant::now();

pub struct HeartbeatStream {
	cooldown: Option<Duration>,
	heartbeat: Option<Duration>,
	next_heartbeat: Option<Delay>,
	last_signal: Option<Instant>,
	streams: VS,
	char: char,
}

impl HeartbeatStream {
	pub fn new(cooldown_sec: Option<u64>, heartbeat_sec: Option<u64>, streams: VS, char: char) -> Self
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
		let mut stream1 = self.streams[0];
		let mut stream2 = self.streams[1];
		let since_last_signal = Instant::now().saturating_duration_since(
			self.last_signal.unwrap_or(heartbeat_imported));

		// Dealing with heartbeat and no heartbeat
		match &mut self.next_heartbeat {
			Some(hb) => {
				Pin::new(hb).poll(cx);
				tokio::spawn(async move {
					tokio::select! {
						Some(char) = stream1.next(), if self.cooldown.map_or(true, |cd| since_last_signal > cd) => {
							self.reset_last_signal_next_heartbeat();
							return Poll::Ready(Some(char))
						},
						Some(char) = stream2.next(), if self.cooldown.map_or(true, |cd| since_last_signal > cd) => {
							self.reset_last_signal_next_heartbeat();
							return Poll::Ready(Some(char))
						},
						_ = hb => {
							self.reset_last_signal_next_heartbeat();
							return Poll::Ready(Some(self.char))
						},
						else => Poll::Ready(None),
					}
				});
			},
			None => {
				tokio::spawn(async move {
					tokio::select! {
						Some(char) = stream1.next(), if self.cooldown.map_or(true, |cd| since_last_signal > cd) => {
							self.reset_last_signal_next_heartbeat();
							return Poll::Ready(Some(char))
						},
						Some(char) = stream2.next(), if self.cooldown.map_or(true, |cd| since_last_signal > cd) => {
							self.reset_last_signal_next_heartbeat();
							return Poll::Ready(Some(char))
						},
						else => Poll::Ready(None),
					}
				});
			}
		}

		Poll::Pending
	}
}
