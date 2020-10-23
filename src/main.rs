use tokio::{
	stream::{Stream, StreamExt},
};
use futures_timer::Delay;
use core::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
	time::Duration,
};
use std::io::{self, Write};
use rand::Rng;

struct IncomingStream {
	min_dur: Duration,
	max_dur: Duration,
	burst: bool,
	burst_max: Option<u32>,
	delay: Option<Delay>,
	char: char
}

impl IncomingStream {
	pub fn new(min: u64, max: u64, burst: bool, burst_max: Option<u32>, char: char) -> Self {
		let mut rs = IncomingStream {
			char,
			burst,
			burst_max,
			min_dur: Duration::from_secs(min),
			max_dur: Duration::from_secs(max),
			delay: None,
		};
		rs.set_next_delay();
		rs
	}

	pub fn new_regular(min: u64, max: u64, char: char) -> Self {
		Self::new(min, max, false, None, char)
	}

	pub fn fixed(sec: u64, char: char) -> Self {
		Self::new(sec, sec, false, None, char)
	}

	pub fn new_burst(min: u64, max: u64, burst_max: u32, char: char) -> Self {
		Self::new(min, max, true, Some(burst_max), char)
	}

	fn set_next_delay(&mut self) {
		let mut rng = rand::thread_rng();
		let [min, max] = [self.min_dur.as_secs(), self.max_dur.as_secs()];
		let next_dur = rng.gen_range(min, max + 1);
		self.delay = Some(Delay::new(Duration::from_secs(next_dur)));
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

#[tokio::main]
pub async fn main() -> Result<(), ()> {
  println!("Running main async");
  let mut rs = IncomingStream::new_regular(3, 8, 'H');

  // how you poll a stream
  while let Some(c) = rs.next().await {
  	print!("{}", c);
  	// you need flushing, because rust print! is buffered
  	io::stdout().flush().expect("should return successfully");
  }

  Ok(())
}
