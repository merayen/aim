use std::ffi::CString;

use pulseaudio_test::{
		pa_context,
		pa_context_connect,
		pa_context_get_state,
		pa_context_new,
		pa_context_set_state_callback,
		pa_context_state_PA_CONTEXT_AUTHORIZING,
		pa_context_state_PA_CONTEXT_CONNECTING,
		pa_context_state_PA_CONTEXT_FAILED,
		pa_context_state_PA_CONTEXT_SETTING_NAME,
		pa_context_state_PA_CONTEXT_TERMINATED,
		pa_context_state_PA_CONTEXT_UNCONNECTED,
		pa_io_event,
		pa_io_event_flags_PA_IO_EVENT_INPUT,
		pa_io_event_flags_t,
		pa_mainloop_api,
		pa_mainloop_get_api,
		pa_mainloop_new,
		pa_mainloop_run,
		pa_signal_init,
		pa_stream,
		pa_stream_get_state,
		pa_stream_state_PA_STREAM_CREATING,
		pa_stream_state_PA_STREAM_FAILED,
		pa_stream_state_PA_STREAM_READY,
		pa_stream_state_PA_STREAM_TERMINATED,
		pa_stream_state_PA_STREAM_UNCONNECTED,
		pa_xfree,
		pa_xstrdup,
};

const	STDIN_FILENO: i32 =	0;	/* Standard input.  */
const	STDOUT_FILENO: i32 =	1;	/* Standard output.  */
const	STDERR_FILENO: i32 =	2;	/* Standard error output.  */


pub extern "C" fn stdin_callback(
			mainloop_api: *mut pa_mainloop_api,
			event: *mut pa_io_event,
			fd: i32,
			flags: u32, // pa_io_event_flags_t
			userdata: *mut std::os::raw::c_void,
) {
	// TODO merayen use pa_stream_get_state() here
	panic!("stdin_callback was called!"); // TODO merayen remove
}

pub extern "C" fn context_state_callback(
	pa_context: *mut pa_context,
	userdata: *mut std::os::raw::c_void,
) {
	if pa_context == std::ptr::null_mut() {
		panic!("Missing context"); // TODO merayen remove
	}
	let state;
	unsafe {
		state = pa_context_get_state(pa_context);
	}

	match state {
		pa_context_state_PA_CONTEXT_UNCONNECTED => {
			println!("pa_context_state_PA_CONTEXT_UNCONNECTED");
		}
		pa_context_state_PA_CONTEXT_CONNECTING => {
			println!("pa_context_state_PA_CONTEXT_CONNECTING");
		}
		pa_context_state_PA_CONTEXT_AUTHORIZING => {
			println!("pa_context_state_PA_CONTEXT_AUTHORIZING");
		}
		pa_context_state_PA_CONTEXT_SETTING_NAME => {
			println!("pa_context_state_PA_CONTEXT_SETTING_NAME");
		}
		pa_context_state_PA_CONTEXT_READY => {
			println!("pa_context_state_PA_CONTEXT_READY");
			let r: i32;
			//pa_stream_set_state_callback(stream, stream_state_callback, std::ptr::null_mut());
			//pa_stream_set_write_callback(stream, stream_write_callback, std::ptr::null_mut());
			//pa_stream_set_read_callback(stream, stream_read_callback, std::ptr::null_mut());
			//pa_stream_set_suspended_callback(stream, stream_suspended_callback, std::ptr::null_mut());
			//pa_stream_set_moved_callback(stream, stream_moved_callback, std::ptr::null_mut());
			//pa_stream_set_underflow_callback(stream, stream_underflow_callback, std::ptr::null_mut());
			//pa_stream_set_overflow_callback(stream, stream_overflow_callback, std::ptr::null_mut());
			//pa_stream_set_started_callback(stream, stream_started_callback, std::ptr::null_mut());
			//pa_stream_set_event_callback(stream, stream_event_callback, std::ptr::null_mut());
			//pa_stream_set_buffer_attr_callback(stream, stream_buffer_attr_callback, std::ptr::null_mut());
		}
		pa_context_state_PA_CONTEXT_FAILED => {
			println!("pa_context_state_PA_CONTEXT_FAILED");
		}
		pa_context_state_PA_CONTEXT_TERMINATED => {
			println!("pa_context_state_PA_CONTEXT_TERMINATED");
		}
		_ => {
			panic!("context_state_callback got unknown state: {}", state); // TODO merayen remove
		}
	}
	println!("context_state_callback({}) was called", state); // TODO merayen remove
}

fn stream_state_callback(
	pa_stream: *mut pa_stream,
	userdata: *mut std::os::raw::c_void,
) {
	if pa_stream == std::ptr::null_mut() {
		panic!("Missing context"); // TODO merayen remove
	}

	let state;
	unsafe {
    state = pa_stream_get_state(pa_stream);
	}

	match state {
    pa_stream_state_PA_STREAM_UNCONNECTED => {
		}
    pa_stream_state_PA_STREAM_CREATING => {
		}
    pa_stream_state_PA_STREAM_READY => {
		}
    pa_stream_state_PA_STREAM_FAILED => {
			panic!("Opening stream failed"); // TODO merayen remove
		}
    pa_stream_state_PA_STREAM_TERMINATED => {
		}
		_ => {
			panic!("stream_state_callback got unknown state: {}", state); // TODO merayen remove
		}
	}
}

fn stream_write_callback() {
}

fn stream_read_callback() {
}

fn stream_suspended_callback() {
}

fn stream_moved_callback() {
}

fn stream_underflow_callback() {
}

fn stream_overflow_callback() {
}

fn stream_started_callback() {
}

fn stream_event_callback() {
}

fn stream_buffer_attr_callback() {
}


fn quit(retval: i32) {
}

fn main() {
	unsafe {
		let stdio_event: *mut std::os::raw::c_void;
		let mainloop = pa_mainloop_new();
		if mainloop.is_null() {
			panic!("Could not create mainloop");
		}

		let mainloop_api: *mut pa_mainloop_api = pa_mainloop_get_api(mainloop);

		if pa_signal_init(mainloop_api) != 0 {
			panic!("pa_signal_init failed");
		}

		let name = CString::new("aim").unwrap().as_ptr();
		let client_name: *mut std::os::raw::c_char = pa_xstrdup(name);
		let stream_name: *mut std::os::raw::c_char = pa_xstrdup(name);

		// TODO merayen implement exit_signal_callback sigusr1_signal_callback?

		// TODO merayen maybe implement signal(SIGPIPE, SIG_IGN);?

		let stdio_event = ((*mainloop_api).io_new.unwrap())(
			mainloop_api,
			STDIN_FILENO,
			pa_io_event_flags_PA_IO_EVENT_INPUT,
			Some(stdin_callback),
			std::ptr::null_mut(),
		);

		if stdio_event == std::ptr::null_mut() {
			panic!("Could not run io_new");
		}

		let context = pa_context_new(mainloop_api, client_name);

		if context == std::ptr::null_mut() {
			panic!("Could not run pa_context_new");
		}

		pa_context_set_state_callback(context, Some(context_state_callback), std::ptr::null_mut());

		if pa_context_connect(context, std::ptr::null(), 0, std::ptr::null()) < 0 {
			panic!("Could not run pa_context_connect");
		}

		pa_mainloop_run(mainloop, &mut 0);

		pa_xfree(client_name as *mut _);
		pa_xfree(stream_name as *mut _);
	}
}
