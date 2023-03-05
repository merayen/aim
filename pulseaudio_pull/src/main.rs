use std::ffi::CString;

#[repr(C)]
struct pa_mainloop_api {
	userdata: *const::std::os::raw::c_void,
	io_new: fn(mainloop_api: *mut pa_mainloop_api),
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
		mainloop: *mut std::os::raw::c_void,
		name: *mut std::os::raw::c_void,
	) -> *const std::os::raw::c_void;

	fn pa_stream_set_write_callback(
		pa_stream: *mut std::os::raw::c_void,
		cb: *mut std::os::raw::c_void,
		user_data: *mut std::os::raw::c_void,
	);

	fn pa_stream_new(
		pa_context: *mut std::os::raw::c_void,
		name: *const std::os::raw::c_void,
		pa_sample_spec: *mut std::os::raw::c_void,
		pa_channel_map: *mut std::os::raw::c_void,
	) -> *const std::os::raw::c_void;

	fn pa_context_get_state(
		pa_context: *mut std::os::raw::c_void,
	) -> i32;
}

const PA_CONTEXT_UNCONNECTED: i32 = 0;    /**< The context hasn't been connected yet */
const PA_CONTEXT_CONNECTING: i32 = 1;     /**< A connection is being established */
const PA_CONTEXT_AUTHORIZING: i32 = 2;    /**< The client is authorizing itself to the daemon */
const PA_CONTEXT_SETTING_NAME: i32 = 3;   /**< The client is passing its application name to the daemon */
const PA_CONTEXT_READY: i32 = 4;          /**< The connection is established, the context is ready to execute operations */
const PA_CONTEXT_FAILED: i32 = 5;         /**< The connection failed or was disconnected */
const PA_CONTEXT_TERMINATED: i32 = 6;      /**< The connection was terminated cleanly */

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

		((*mainloop_api).io_new)(mainloop_api);

		pa_xfree(client_name);
		pa_xfree(stream_name);
	}
}
