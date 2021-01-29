use std::os::unix::io::AsRawFd;
use std::io::Read;

// Takes care of input
pub struct Input {
}

impl Input {
	pub fn new() -> Input {
		let mut termios = termios::Termios::from_fd(std::io::stdin().as_raw_fd()).unwrap();
		termios::cfmakeraw(&mut termios);
		//termios::tcsetattr(std::io::stdin().as_raw_fd(), termios::TCSAFLUSH, &termios).unwrap();

		//for i in 0..10 {
		//	let mut noe = [0u8; 1];
		//	std::io::stdin().read(&mut noe);
		//	//println!("{:?}", noe);
		//	if noe[0] == 3 {
		//		break;
		//	}
		//}

		Input {}
	}
}
