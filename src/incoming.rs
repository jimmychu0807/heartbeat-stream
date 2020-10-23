use tokio::{
	stream::{Stream},
};
use futures_timer::Delay;
use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
	time::Duration,
};
use rand::Rng;

pub struct IncomingStream {
	min_dur: Duration,
	max_dur: Duration,
	burst_max: Option<u32>,
	burst_cnt: u32,
	delay: Option<Delay>,
	char: char
}

impl IncomingStream {
	pub fn new(min: u64, max: u64, burst_max: Option<u32>, char: char) -> Self {
		let mut rs = IncomingStream {
			char,
			burst_max,
			burst_cnt: 0,
			min_dur: Duration::from_secs(min),
			max_dur: Duration::from_secs(max),
			delay: None,
		};
		rs.set_next_delay();
		rs
	}

	pub fn new_regular(min: u64, max: u64, char: char) -> Self {
		Self::new(min, max, None, char)
	}

	pub fn fixed(sec: u64, char: char) -> Self {
		Self::new(sec, sec, None, char)
	}

	pub fn new_burst(min: u64, max: u64, burst_max: u32, char: char) -> Self {
		Self::new(min, max, Some(burst_max), char)
	}

	fn set_next_delay(&mut self) {
		let set_delay = |s: &mut Self| {
			let mut rng = rand::thread_rng();
			let [min, max] = [s.min_dur.as_secs(), s.max_dur.as_secs()];
			let next_dur = rng.gen_range(min, max + 1);
			s.burst_cnt = 0;
			s.delay = Some(Delay::new(Duration::from_secs(next_dur)));
		};

		match self.burst_max {
			None => set_delay(self),
			Some(burst_max) if burst_max == self.burst_cnt => set_delay(self),
			_ => {
				// Handling burst
				self.burst_cnt += 1;
				self.delay = Some(Delay::new(Duration::from_millis(50)));
			}
		}
	}
}

impl Stream for IncomingStream {
	// TODO: can you enhance on this to pass by from beginning
	type Item = char;
	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		match &mut self.delay {
			None => Poll::Ready(None),
			Some(delay) => {
				match Pin::new(delay).poll(cx) {
					Poll::Pending => Poll::Pending,
					Poll::Ready(_) => {
						// set the delay for next random wakeup time
						self.set_next_delay();
						Poll::Ready(Some(self.char))
					}
				}
			}
		}
	}
}
