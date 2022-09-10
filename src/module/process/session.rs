//! Session handling for voices of nodes
// TODO merayen remove this module, we are simplifying
use crate::nodes::common;

pub struct Session {
	/// Lists active voices. Matches the voice indices
	pub active_voices: Vec<bool>,
}

impl Session {
	pub fn new(max_voice_count: u16) -> Self {
		let mut active_voices = Vec::with_capacity(max_voice_count.into());
		for _ in 0..max_voice_count {
			active_voices.push(false);
		}

		Session { active_voices }
	}
}

/// Create a new voice for a list of ports
///
/// Returns the voice_index of the created/re-used voice.
pub fn create_voice(session: &mut Session, ports: &mut Vec<common::Ports>, buffer_size: usize) -> Result<usize, ()> {
	let voice_index = find_empty_voice(ports)?;

	for port in ports {
		for mut outlet in &mut port.outlets.values() {
			if let Some(audio) = &outlet.audio {
				//audio.push();
			}
			else if let signal = outlet.signal.is_some() {
			} else if let midi = outlet.midi.is_some() {
				unimplemented!("Not implemented"); // TODO merayen
			} else {
				unimplemented!("Not implemented"); // TODO merayen
			}
		}
	}

	Ok(voice_index)
}

/// Find a previously used voice that it no more in use
fn find_empty_voice(ports: &mut Vec<common::Ports>) -> Result<usize, ()> {
	let mut voice_index = -1isize;

	for port in ports {
	}

	Ok(voice_index as usize)
}
