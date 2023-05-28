use std::ffi::CString;

const	STDIN_FILENO: i32 =	0;	/* Standard input.  */
const	STDOUT_FILENO: i32 =	1;	/* Standard output.  */
const	STDERR_FILENO: i32 =	2;	/* Standard error output.  */


mod pa_context_flags_t {
	pub const PA_CONTEXT_NOFLAGS: u32 = 0x0000;
	pub const PA_CONTEXT_NOAUTOSPAWN: u32 = 0x0001;
	pub const PA_CONTEXT_NOFAIL: u32 = 0x0002;
}

mod pa_io_event_flags_t {
	pub const PA_IO_EVENT_NULL: i32 = 0; /* No event */
	pub const PA_IO_EVENT_INPUT: i32 = 1; /* Input event */
	pub const PA_IO_EVENT_OUTPUT: i32 = 2; /* Output event */
	pub const PA_IO_EVENT_HANGUP: i32 = 4; /* Hangup event */
	pub const PA_IO_EVENT_ERROR: i32 = 8; /* Error event */
}

mod pa_stream_state {
    pub const PA_STREAM_UNCONNECTED: i32 = 0; /* The stream is not yet connected to any sink or source */
    pub const PA_STREAM_CREATING: i32 = 1; /* The stream is being created */
    pub const PA_STREAM_READY: i32 = 2; /* The stream is established, you may pass audio data to it now */
    pub const PA_STREAM_FAILED: i32 = 3; /* An error occurred that made the stream invalid */
    pub const PA_STREAM_TERMINATED: i32 = 4; /* i32 = 0< The stream has been terminated cleanly */
}

#[repr(C)]
struct pa_mainloop_api {
	userdata: *const::std::os::raw::c_void,
	io_new: fn(
		mainloop_api: *mut pa_mainloop_api,
		fd: std::os::raw::c_int,
		events: i32,
		cb: fn(
			mainloop_api: *mut pa_mainloop_api,
			event: *mut std::os::raw::c_void,
			fd: i32,
			flags: i32, // pa_io_event_flags_t
			userdata: *mut std::os::raw::c_void,
		),
		*const std::os::raw::c_void,
	) -> *mut std::os::raw::c_void,
}

struct pa_spawn_api {
	/** Is called just before the fork in the parent process. May be
	* NULL. */
	prefork: fn(),

	/** Is called immediately after the fork in the parent
	* process. May be NULL.*/
	postfork: fn(),

	/** Is called immediately after the fork in the child
	* process. May be NULL. It is not safe to close all file
	* descriptors in this function unconditionally, since a UNIX
	* socket (created using socketpair()) is passed to the new
	* process. */
	atfork: fn(),
}

extern {
	fn pa_xstrdup(
		string: *const std::os::raw::c_char,
	) -> *mut std::os::raw::c_void;

	fn pa_xmalloc(
		size: u32,
	) -> *mut std::os::raw::c_void;

	fn pa_xfree(
		reference: *mut std::os::raw::c_void,
	);

	fn pa_mainloop_new(
	) -> *mut std::os::raw::c_void;

	fn pa_mainloop_get_api(
		pa_mainloop: *mut std::os::raw::c_void,
	) -> *mut pa_mainloop_api;

	fn pa_signal_init(
		mainloop_api: *mut pa_mainloop_api,
	) -> std::os::raw::c_int;

	fn pa_context_new(
		mainloop: *mut pa_mainloop_api,
		name: *mut std::os::raw::c_void,
	) -> *mut std::os::raw::c_void;

	fn pa_stream_new(
		pa_context: *mut std::os::raw::c_void,
		name: *const std::os::raw::c_void,
		pa_sample_spec: *mut std::os::raw::c_void,
		pa_channel_map: *mut std::os::raw::c_void,
	) -> *const std::os::raw::c_void;

	fn pa_context_get_state(
		pa_context: *mut std::os::raw::c_void,
	) -> i32;

	fn pa_context_set_state_callback(
		pa_context: *mut std::os::raw::c_void,
		cb: fn(
			pa_context: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		)
	);

	fn pa_context_connect(
		pa_context: *mut std::os::raw::c_void,
		server: *const std::os::raw::c_char,
		flags: std::os::raw::c_uint, // pa_context_flags_t
		api: *const pa_spawn_api,
	) -> std::os::raw::c_int;

	fn pa_mainloop_run(
		mainloop: *mut std::os::raw::c_void,
		retval: *mut std::os::raw::c_int,
	);

	fn pa_stream_get_state(
		pa_stream: *const std::os::raw::c_void,
	) -> std::os::raw::c_int;

	fn pa_stream_set_state_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_write_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			nbytes: usize,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_read_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			nbytes: usize,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_suspended_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_moved_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_underflow_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_overflow_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_started_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_event_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			name: *const std::os::raw::c_char,
			pa_proplist: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_buffer_attr_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_set_latency_update_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: fn(
			pa_stream: *mut std::os::raw::c_void,
			userdata: *mut std::os::raw::c_void,
		),
		user_data: *mut std::os::raw::c_void,
	);
}

mod pa_context_state {
	pub const PA_CONTEXT_UNCONNECTED: i32 = 0;    /* The context hasn't been connected yet */
	pub const PA_CONTEXT_CONNECTING: i32 = 1;     /* A connection is being established */
	pub const PA_CONTEXT_AUTHORIZING: i32 = 2;    /* The client is authorizing itself to the daemon */
	pub const PA_CONTEXT_SETTING_NAME: i32 = 3;   /* The client is passing its application name to the daemon */
	pub const PA_CONTEXT_READY: i32 = 4;          /* The connection is established, the context is ready to execute operations */
	pub const PA_CONTEXT_FAILED: i32 = 5;         /* The connection failed or was disconnected */
	pub const PA_CONTEXT_TERMINATED: i32 = 6;      /* The connection was terminated cleanly */
}

fn stdin_callback(
			mainloop_api: *mut pa_mainloop_api,
			event: *mut std::os::raw::c_void,
			fd: i32,
			flags: std::os::raw::c_int, // pa_io_event_flags_t
			userdata: *mut std::os::raw::c_void,
) {
	// TODO merayen use pa_stream_get_state() here
	panic!("stdin_callback was called!"); // TODO merayen remove
}

fn context_state_callback(
	pa_context: *mut std::os::raw::c_void,
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
		pa_context_state::PA_CONTEXT_UNCONNECTED => {
		}
		pa_context_state::PA_CONTEXT_CONNECTING => {
		}
		pa_context_state::PA_CONTEXT_AUTHORIZING => {
		}
		pa_context_state::PA_CONTEXT_SETTING_NAME => {
		}
		pa_context_state::PA_CONTEXT_READY => {
			// TODO merayen handle the other things above this
			pa_stream_set_state_callback(stream, stream_state_callback, NULL);
			pa_stream_set_write_callback(stream, stream_write_callback, NULL);
			pa_stream_set_read_callback(stream, stream_read_callback, NULL);
			pa_stream_set_suspended_callback(stream, stream_suspended_callback, NULL);
			pa_stream_set_moved_callback(stream, stream_moved_callback, NULL);
			pa_stream_set_underflow_callback(stream, stream_underflow_callback, NULL);
			pa_stream_set_overflow_callback(stream, stream_overflow_callback, NULL);
			pa_stream_set_started_callback(stream, stream_started_callback, NULL);
			pa_stream_set_event_callback(stream, stream_event_callback, NULL);
			pa_stream_set_buffer_attr_callback(stream, stream_buffer_attr_callback, NULL);
		}
		pa_context_state::PA_CONTEXT_FAILED => {
		}
		pa_context_state::PA_CONTEXT_TERMINATED => {
		}
		_ => {
			panic!("context_state_callback got unknown state: {}", state); // TODO merayen remove
		}
	}
	println!("context_state_callback({}) was called", state); // TODO merayen remove
}

fn stream_state_callback(
	pa_stream: *mut std::os::raw::c_void,
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
    pa_stream_state::PA_STREAM_UNCONNECTED => {
		}
    pa_stream_state::PA_STREAM_CREATING => {
		}
    pa_stream_state::PA_STREAM_READY => {
		}
    pa_stream_state::PA_STREAM_FAILED => {
			panic!("Opening stream failed"); // TODO merayen remove
		}
    pa_stream_state::PA_STREAM_TERMINATED => {
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
		let client_name = pa_xstrdup(name);
		let stream_name = pa_xstrdup(name);

		// TODO merayen implement exit_signal_callback sigusr1_signal_callback?

		// TODO merayen maybe implement signal(SIGPIPE, SIG_IGN);?

		let stdio_event = ((*mainloop_api).io_new)(
			mainloop_api,
			STDIN_FILENO,
			pa_io_event_flags_t::PA_IO_EVENT_INPUT,
			stdin_callback,
			std::ptr::null(),
		);

		if stdio_event == std::ptr::null_mut() {
			panic!("Could not run io_new");
		}

		let context = pa_context_new(mainloop_api, client_name);

		if context == std::ptr::null_mut() {
			panic!("Could not run pa_context_new");
		}

		pa_context_set_state_callback(context, context_state_callback);

		if pa_context_connect(context, std::ptr::null(), 0, std::ptr::null()) < 0 {
			panic!("Could not run pa_context_connect");
		}

		pa_mainloop_run(mainloop, &mut 0);

		pa_xfree(client_name);
		pa_xfree(stream_name);
	}
}
