use tokio::{
	stream::{StreamExt},
};
use std::io::{self, Write};

mod incoming;
use incoming::IncomingStream;

mod heartbeat;
use heartbeat::HeartbeatStream;

#[tokio::main]
pub async fn main() -> Result<(), ()> {
  println!("Running main async");
  // let mut rs = IncomingStream::new_regular(3, 8, 'H');
  let mut rs = IncomingStream::new_burst(3, 8, 4, 'H');

  // how you poll a stream
  while let Some(c) = rs.next().await {
  	print!("{}", c);
  	// you need flushing, because rust print! is buffered
  	io::stdout().flush().expect("should return successfully");
  }

  Ok(())
}
