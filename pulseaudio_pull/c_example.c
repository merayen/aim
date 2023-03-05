#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <signal.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <getopt.h>
#include <fcntl.h>
#include <locale.h>

#include <pulse/pulseaudio.h>
#include <pulse/rtclock.h>

#define TIME_EVENT_USEC 50000

#define CLEAR_LINE "\x1B[K"

static enum { RECORD, PLAYBACK } mode = PLAYBACK;

static pa_context *context = NULL;
static pa_stream *stream = NULL;
static pa_mainloop_api *mainloop_api = NULL;

static void *buffer = NULL;
static size_t buffer_length = 0, buffer_index = 0;

static pa_io_event* stdio_event = NULL;

static char *stream_name = NULL, *client_name = NULL, *device = NULL;

static int verbose = 0;
static pa_volume_t volume = PA_VOLUME_NORM;
static int volume_is_set = 0;

static pa_sample_spec sample_spec = {
    .format = PA_SAMPLE_S16LE,
    .rate = 44100,
    .channels = 2
};

static pa_channel_map channel_map;
static int channel_map_set = 0;

static pa_stream_flags_t flags = 0;

static size_t latency = 0, process_time=0;

/* A shortcut for terminating the application */
static void quit(int ret) {
    assert(mainloop_api);
    mainloop_api->quit(mainloop_api, ret);
}

/* Write some data to the stream */
static void do_stream_write(size_t length) {
    size_t l;
    assert(length);

    if (!buffer || !buffer_length)
        return;

    l = length;
    if (l > buffer_length)
        l = buffer_length;

    if (pa_stream_write(stream, (uint8_t*) buffer + buffer_index, l, NULL, 0, PA_SEEK_RELATIVE) < 0) {
        fprintf(stderr, "pa_stream_write() failed: %s\n", pa_strerror(pa_context_errno(context)));
        quit(1);
        return;
    }

    buffer_length -= l;
    buffer_index += l;

    if (!buffer_length) {
        pa_xfree(buffer);
        buffer = NULL;
        buffer_index = buffer_length = 0;
    }
}

/* This is called whenever new data may be written to the stream */
static void stream_write_callback(pa_stream *s, size_t length, void *userdata) {
    assert(s);
    assert(length > 0);

    if (stdio_event)
        mainloop_api->io_enable(stdio_event, PA_IO_EVENT_INPUT);

    if (!buffer)
        return;

    do_stream_write(length);
}

/* This is called whenever new data may is available */
static void stream_read_callback(pa_stream *s, size_t length, void *userdata) {
    const void *data;
    assert(s);
    assert(length > 0);

    if (stdio_event)
        mainloop_api->io_enable(stdio_event, PA_IO_EVENT_OUTPUT);

    if (pa_stream_peek(s, &data, &length) < 0) {
        fprintf(stderr, "pa_stream_peek() failed: %s\n", pa_strerror(pa_context_errno(context)));
        quit(1);
        return;
    }

    assert(data);
    assert(length > 0);

    if (buffer) {
        buffer = pa_xrealloc(buffer, buffer_length + length);
        memcpy((uint8_t*) buffer + buffer_length, data, length);
        buffer_length += length;
    } else {
        buffer = pa_xmalloc(length);
        memcpy(buffer, data, length);
        buffer_length = length;
        buffer_index = 0;
    }

    pa_stream_drop(s);
}

/* This routine is called whenever the stream state changes */
static void stream_state_callback(pa_stream *s, void *userdata) {  // TODO merayen implement stream_state_callback
    assert(s);

    switch (pa_stream_get_state(s)) {  // TODO merayen implement this
        case PA_STREAM_CREATING:
        case PA_STREAM_TERMINATED:
            break;

        case PA_STREAM_READY:
            if (verbose) {
                const pa_buffer_attr *a;
                char cmt[PA_CHANNEL_MAP_SNPRINT_MAX], sst[PA_SAMPLE_SPEC_SNPRINT_MAX];

                fprintf(stderr, "Stream successfully created.\n");

                if (!(a = pa_stream_get_buffer_attr(s)))
                    fprintf(stderr, "pa_stream_get_buffer_attr() failed: %s\n", pa_strerror(pa_context_errno(pa_stream_get_context(s))));
                else {

                    if (mode == PLAYBACK)
                        fprintf(stderr, "Buffer metrics: maxlength=%u, tlength=%u, prebuf=%u, minreq=%u\n", a->maxlength, a->tlength, a->prebuf, a->minreq);
                    else {
                        assert(mode == RECORD);
                        fprintf(stderr, "Buffer metrics: maxlength=%u, fragsize=%u\n", a->maxlength, a->fragsize);
                    }
                }

                fprintf(stderr, "Using sample spec '%s', channel map '%s'.\n",
                        pa_sample_spec_snprint(sst, sizeof(sst), pa_stream_get_sample_spec(s)),
                        pa_channel_map_snprint(cmt, sizeof(cmt), pa_stream_get_channel_map(s)));

                fprintf(stderr, "Connected to device %s (%u, %ssuspended).\n",
                        pa_stream_get_device_name(s),
                        pa_stream_get_device_index(s),
                        pa_stream_is_suspended(s) ? "" : "not ");
            }

            break;

        case PA_STREAM_FAILED:
        default:
            fprintf(stderr, "Stream error: %s\n", pa_strerror(pa_context_errno(pa_stream_get_context(s))));
            quit(1);
    }
}

static void stream_suspended_callback(pa_stream *s, void *userdata) {
    assert(s);

    if (verbose) {
        if (pa_stream_is_suspended(s))
            fprintf(stderr, "Stream device suspended.%s \n", CLEAR_LINE);
        else
            fprintf(stderr, "Stream device resumed.%s \n", CLEAR_LINE);
    }
}

static void stream_underflow_callback(pa_stream *s, void *userdata) {
    assert(s);

    if (verbose)
        fprintf(stderr, "Stream underrun.%s \n",  CLEAR_LINE);
}

static void stream_overflow_callback(pa_stream *s, void *userdata) {
    assert(s);

    if (verbose)
        fprintf(stderr, "Stream overrun.%s \n", CLEAR_LINE);
}

static void stream_started_callback(pa_stream *s, void *userdata) {
    assert(s);

    if (verbose)
        fprintf(stderr, "Stream started.%s \n", CLEAR_LINE);
}

static void stream_moved_callback(pa_stream *s, void *userdata) {
    assert(s);

    if (verbose)
        fprintf(stderr, "Stream moved to device %s (%u, %ssuspended).%s \n", pa_stream_get_device_name(s), pa_stream_get_device_index(s), pa_stream_is_suspended(s) ? "" : "not ",  CLEAR_LINE);
}

static void stream_buffer_attr_callback(pa_stream *s, void *userdata) {
    assert(s);

    if (verbose)
        fprintf(stderr, "Stream buffer attributes changed.%s \n",  CLEAR_LINE);
}

static void stream_event_callback(pa_stream *s, const char *name, pa_proplist *pl, void *userdata) {
    char *t;

    assert(s);
    assert(name);
    assert(pl);

    t = pa_proplist_to_string_sep(pl, ", ");
    fprintf(stderr, "Got event '%s', properties '%s'\n", name, t);
    pa_xfree(t);
}

/* This is called whenever the context status changes */
static void context_state_callback(pa_context *c, void *userdata) {
    assert(c);

    switch (pa_context_get_state(c)) {
        case PA_CONTEXT_CONNECTING:
        case PA_CONTEXT_AUTHORIZING:
        case PA_CONTEXT_SETTING_NAME:
            break;

        case PA_CONTEXT_READY: {
            int r;
            pa_buffer_attr buffer_attr;

            assert(c);
            assert(!stream);

            if (verbose)
                fprintf(stderr, "Connection established.%s \n", CLEAR_LINE);

            if (!(stream = pa_stream_new(c, stream_name, &sample_spec, channel_map_set ? &channel_map : NULL))) {
                fprintf(stderr, "pa_stream_new() failed: %s\n", pa_strerror(pa_context_errno(c)));
                goto fail;
            }

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

            if (latency > 0) {
                memset(&buffer_attr, 0, sizeof(buffer_attr));
                buffer_attr.tlength = (uint32_t) latency;
                buffer_attr.minreq = (uint32_t) process_time;
                buffer_attr.maxlength = (uint32_t) -1;
                buffer_attr.prebuf = (uint32_t) -1;
                buffer_attr.fragsize = (uint32_t) latency;
                flags |= PA_STREAM_ADJUST_LATENCY;
            }

            if (mode == PLAYBACK) {
                pa_cvolume cv;
                if ((r = pa_stream_connect_playback(stream, device, latency > 0 ? &buffer_attr : NULL, flags, volume_is_set ? pa_cvolume_set(&cv, sample_spec.channels, volume) : NULL, NULL)) < 0) {
                    fprintf(stderr, "pa_stream_connect_playback() failed: %s\n", pa_strerror(pa_context_errno(c)));
                    goto fail;
                }

            } else {
                if ((r = pa_stream_connect_record(stream, device, latency > 0 ? &buffer_attr : NULL, flags)) < 0) {
                    fprintf(stderr, "pa_stream_connect_record() failed: %s\n", pa_strerror(pa_context_errno(c)));
                    goto fail;
                }
            }

            break;
        }

        case PA_CONTEXT_TERMINATED:
            quit(0);
            break;

        case PA_CONTEXT_FAILED:
        default:
            fprintf(stderr, "Connection failure: %s\n", pa_strerror(pa_context_errno(c)));
            goto fail;
    }

    return;

fail:
    quit(1);

}

/* Connection draining complete */
static void context_drain_complete(pa_context*c, void *userdata) {
    pa_context_disconnect(c);
}

/* Stream draining complete */
static void stream_drain_complete(pa_stream*s, int success, void *userdata) {

    if (!success) {
        fprintf(stderr, "Failed to drain stream: %s\n", pa_strerror(pa_context_errno(context)));
        quit(1);
    }

    if (verbose)
        fprintf(stderr, "Playback stream drained.\n");

    pa_stream_disconnect(stream);
    pa_stream_unref(stream);
    stream = NULL;

    if (!pa_context_drain(context, context_drain_complete, NULL))
        pa_context_disconnect(context);
    else {
        if (verbose)
            fprintf(stderr, "Draining connection to server.\n");
    }
}

/* New data on STDIN **/
static void stdin_callback(pa_mainloop_api*a, pa_io_event *e, int fd, pa_io_event_flags_t f, void *userdata) {
    size_t l, w = 0;
    ssize_t r;

    assert(a == mainloop_api);
    assert(e);
    assert(stdio_event == e);

    if (buffer) {
        mainloop_api->io_enable(stdio_event, PA_IO_EVENT_NULL);
        return;
    }

    if (!stream || pa_stream_get_state(stream) != PA_STREAM_READY || !(l = w = pa_stream_writable_size(stream)))
        l = 4096;

    buffer = pa_xmalloc(l);

    if ((r = read(fd, buffer, l)) <= 0) {
        if (r == 0) {
            if (verbose)
                fprintf(stderr, "Got EOF.\n");

            if (stream) {
                pa_operation *o;

                if (!(o = pa_stream_drain(stream, stream_drain_complete, NULL))) {
                    fprintf(stderr, "pa_stream_drain(): %s\n", pa_strerror(pa_context_errno(context)));
                    quit(1);
                    return;
                }

                pa_operation_unref(o);
            } else
                quit(0);

        } else {
            fprintf(stderr, "read() failed: %s\n", strerror(errno));
            quit(1);
        }

        mainloop_api->io_free(stdio_event);
        stdio_event = NULL;
        return;
    }

    buffer_length = (uint32_t) r;
    buffer_index = 0;

    if (w)
        do_stream_write(w);
}

/* Some data may be written to STDOUT */
static void stdout_callback(pa_mainloop_api*a, pa_io_event *e, int fd, pa_io_event_flags_t f, void *userdata) {
    ssize_t r;

    assert(a == mainloop_api);
    assert(e);
    assert(stdio_event == e);

    if (!buffer) {
        mainloop_api->io_enable(stdio_event, PA_IO_EVENT_NULL);
        return;
    }

    assert(buffer_length);

    if ((r = write(fd, (uint8_t*) buffer+buffer_index, buffer_length)) <= 0) {
        fprintf(stderr, "write() failed: %s\n", strerror(errno));
        quit(1);

        mainloop_api->io_free(stdio_event);
        stdio_event = NULL;
        return;
    }

    buffer_length -= (uint32_t) r;
    buffer_index += (uint32_t) r;

    if (!buffer_length) {
        pa_xfree(buffer);
        buffer = NULL;
        buffer_length = buffer_index = 0;
    }
}

/* UNIX signal to quit recieved */
static void exit_signal_callback(pa_mainloop_api*m, pa_signal_event *e, int sig, void *userdata) {
    if (verbose)
        fprintf(stderr, "Got signal, exiting.\n");
    quit(0);
}

/* Show the current latency */
static void stream_update_timing_callback(pa_stream *s, int success, void *userdata) {
    pa_usec_t l, usec;
    int negative = 0;

    assert(s);

    if (!success ||
        pa_stream_get_time(s, &usec) < 0 ||
        pa_stream_get_latency(s, &l, &negative) < 0) {
        fprintf(stderr, "Failed to get latency: %s\n", pa_strerror(pa_context_errno(context)));
        quit(1);
        return;
    }

    fprintf(stderr, "Time: %0.3f sec; Latency: %0.0f usec.  \r",
            (float) usec / 1000000,
            (float) l * (negative?-1.0f:1.0f));
}

/* Someone requested that the latency is shown */
static void sigusr1_signal_callback(pa_mainloop_api*m, pa_signal_event *e, int sig, void *userdata) {

    if (!stream)
        return;

    pa_operation_unref(pa_stream_update_timing_info(stream, stream_update_timing_callback, NULL));
}

static void time_event_callback(pa_mainloop_api *m, pa_time_event *e, const struct timeval *tv, void *userdata) {
    if (stream && pa_stream_get_state(stream) == PA_STREAM_READY) {
        pa_operation *o;
        if (!(o = pa_stream_update_timing_info(stream, stream_update_timing_callback, NULL)))
            fprintf(stderr, "pa_stream_update_timing_info() failed: %s\n", pa_strerror(pa_context_errno(context)));
        else
            pa_operation_unref(o);
    }

    struct timeval now;
    gettimeofday(&now, NULL);
    pa_timeval_add(&now, TIME_EVENT_USEC);
    m->time_restart(e, &now);
}


int main(int argc, char *argv[]) {
    pa_mainloop* m = NULL;
    int ret = 1, r, c;
    char *bn, *server = NULL;
    pa_time_event *time_event = NULL;

    static const struct option long_options[] = {
        {"record",       0, NULL, 'r'},
        {"playback",     0, NULL, 'p'},
        {"device",       1, NULL, 'd'},
        {"server",       1, NULL, 's'},
        {"client-name",  1, NULL, 'n'},
        {"stream-name",  1, NULL, ARG_STREAM_NAME},
        {"version",      0, NULL, ARG_VERSION},
        {"help",         0, NULL, 'h'},
        {"verbose",      0, NULL, 'v'},
        {"volume",       1, NULL, ARG_VOLUME},
        {"rate",         1, NULL, ARG_SAMPLERATE},
        {"format",       1, NULL, ARG_SAMPLEFORMAT},
        {"channels",     1, NULL, ARG_CHANNELS},
        {"channel-map",  1, NULL, ARG_CHANNELMAP},
        {"fix-format",   0, NULL, ARG_FIX_FORMAT},
        {"fix-rate",     0, NULL, ARG_FIX_RATE},
        {"fix-channels", 0, NULL, ARG_FIX_CHANNELS},
        {"no-remap",     0, NULL, ARG_NO_REMAP},
        {"no-remix",     0, NULL, ARG_NO_REMIX},
        {"latency",      1, NULL, ARG_LATENCY},
        {"process-time", 1, NULL, ARG_PROCESS_TIME},
        {NULL,           0, NULL, 0}
    };

    setlocale(LC_ALL, "");

    if (!(bn = strrchr(argv[0], '/')))
        bn = argv[0];
    else
        bn++;

    if (strstr(bn, "rec") || strstr(bn, "mon"))
        mode = RECORD;
    else if (strstr(bn, "cat") || strstr(bn, "play"))
        mode = PLAYBACK;

    while ((c = getopt_long(argc, argv, "rpd:s:n:hv", long_options, NULL)) != -1) {

        switch (c) {
            case 'h' :
                help(bn);
                ret = 0;
                goto quit;

            case ARG_VERSION:
//                printf("pacat %s\nCompiled with libpulse %s\nLinked with libpulse %s\n", PACKAGE_VERSION, pa_get_headers_version(), pa_get_library_version());
                ret = 0;
                goto quit;

            case 'r':
                mode = RECORD;
                break;

            case 'p':
                mode = PLAYBACK;
                break;

            case 'd':
                pa_xfree(device);
                device = pa_xstrdup(optarg);
                break;

            case 's':
                pa_xfree(server);
                server = pa_xstrdup(optarg);
                break;

            case 'n':
                pa_xfree(client_name);
                client_name = pa_xstrdup(optarg);
                break;

            case ARG_STREAM_NAME:
                pa_xfree(stream_name);
                stream_name = pa_xstrdup(optarg);
                break;

            case 'v':
                verbose = 1;
                break;

            case ARG_VOLUME: {
                int v = atoi(optarg);
                volume = v < 0 ? 0U : (pa_volume_t) v;
                volume_is_set = 1;
                break;
            }

            case ARG_CHANNELS:
                sample_spec.channels = (uint8_t) atoi(optarg);
                break;

            case ARG_SAMPLEFORMAT:
                sample_spec.format = pa_parse_sample_format(optarg);
                break;

            case ARG_SAMPLERATE:
                sample_spec.rate = (uint32_t) atoi(optarg);
                break;

            case ARG_CHANNELMAP:
                if (!pa_channel_map_parse(&channel_map, optarg)) {
                    fprintf(stderr, "Invalid channel map '%s'\n", optarg);
                    goto quit;
                }

                channel_map_set = 1;
                break;

            case ARG_FIX_CHANNELS:
                flags |= PA_STREAM_FIX_CHANNELS;
                break;

            case ARG_FIX_RATE:
                flags |= PA_STREAM_FIX_RATE;
                break;

            case ARG_FIX_FORMAT:
                flags |= PA_STREAM_FIX_FORMAT;
                break;

            case ARG_NO_REMIX:
                flags |= PA_STREAM_NO_REMIX_CHANNELS;
                break;

            case ARG_NO_REMAP:
                flags |= PA_STREAM_NO_REMAP_CHANNELS;
                break;

            case ARG_LATENCY:
                if (((latency = (size_t) atoi(optarg))) <= 0) {
                    fprintf(stderr, "Invalid latency specification '%s'\n", optarg);
                    goto quit;
                }
                break;

            case ARG_PROCESS_TIME:
                if (((process_time = (size_t) atoi(optarg))) <= 0) {
                    fprintf(stderr, "Invalid process time specification '%s'\n", optarg);
                    goto quit;
                }
                break;

            default:
                goto quit;
        }
    }

    if (!pa_sample_spec_valid(&sample_spec)) {
        fprintf(stderr, "Invalid sample specification\n");
        goto quit;
    }

    if (channel_map_set && pa_channel_map_compatible(&channel_map, &sample_spec)) {
        fprintf(stderr, "Channel map doesn't match sample specification\n");
        goto quit;
    }

    if (verbose) {
        char t[PA_SAMPLE_SPEC_SNPRINT_MAX];
        pa_sample_spec_snprint(t, sizeof(t), &sample_spec);
        fprintf(stderr, "Opening a %s stream with sample specification '%s'.\n", mode == RECORD ? "recording" : "playback", t);
    }

    if (!(optind >= argc)) {
        if (optind+1 == argc) {
            int fd;

            if ((fd = open(argv[optind], mode == PLAYBACK ? O_RDONLY : O_WRONLY|O_TRUNC|O_CREAT, 0666)) < 0) {
                fprintf(stderr, "open(): %s\n", strerror(errno));
                goto quit;
            }

            if (dup2(fd, mode == PLAYBACK ? 0 : 1) < 0) {
                fprintf(stderr, "dup2(): %s\n", strerror(errno));
                goto quit;
            }

            close(fd);

            if (!stream_name)
                stream_name = pa_xstrdup(argv[optind]);

        } else {
            fprintf(stderr, "Too many arguments.\n");
            goto quit;
        }
    }

    if (!client_name)
        client_name = pa_xstrdup(bn);

    if (!stream_name)
        stream_name = pa_xstrdup(client_name);

    /* Set up a new main loop */
    if (!(m = pa_mainloop_new())) {
        fprintf(stderr, "pa_mainloop_new() failed.\n");
        goto quit;
    }

    mainloop_api = pa_mainloop_get_api(m);

    r = pa_signal_init(mainloop_api);
    assert(r == 0);
    pa_signal_new(SIGINT, exit_signal_callback, NULL);
    pa_signal_new(SIGTERM, exit_signal_callback, NULL);
#ifdef SIGUSR1
    pa_signal_new(SIGUSR1, sigusr1_signal_callback, NULL);
#endif
#ifdef SIGPIPE
    signal(SIGPIPE, SIG_IGN);
#endif

    if (!(stdio_event = mainloop_api->io_new(mainloop_api,
                                             mode == PLAYBACK ? STDIN_FILENO : STDOUT_FILENO,
                                             mode == PLAYBACK ? PA_IO_EVENT_INPUT : PA_IO_EVENT_OUTPUT,
                                             mode == PLAYBACK ? stdin_callback : stdout_callback, NULL))) {
        fprintf(stderr, "io_new() failed.\n");
        goto quit;
    }

    /* Create a new connection context */
    if (!(context = pa_context_new(mainloop_api, client_name))) {
        fprintf(stderr, "pa_context_new() failed.\n");
        goto quit;
    }

    pa_context_set_state_callback(context, context_state_callback, NULL);

    /* Connect the context */
    if (pa_context_connect(context, server, 0, NULL) < 0) {
        fprintf(stderr, "pa_context_connect() failed: %s\n", pa_strerror(pa_context_errno(context)));
        goto quit;
    }

    if (verbose) {
        struct timeval now;
        gettimeofday(&now, NULL);
        pa_timeval_add(&now, TIME_EVENT_USEC);
        if (!(time_event = mainloop_api->time_new(mainloop_api, &now, time_event_callback, NULL))) {
            fprintf(stderr, "time_new() failed.\n");
            goto quit;
        }
    }

    /* Run the main loop */
    if (pa_mainloop_run(m, &ret) < 0) {
        fprintf(stderr, "pa_mainloop_run() failed.\n");
        goto quit;
    }

quit:
    if (stream)
        pa_stream_unref(stream);

    if (context)
        pa_context_unref(context);

    if (stdio_event) {
        assert(mainloop_api);
        mainloop_api->io_free(stdio_event);
    }

    if (time_event) {
        assert(mainloop_api);
        mainloop_api->time_free(time_event);
    }

    if (m) {
        pa_signal_done();
        pa_mainloop_free(m);
    }

    pa_xfree(buffer);

    pa_xfree(server);
    pa_xfree(device);
    pa_xfree(client_name);
    pa_xfree(stream_name);

    return ret;
}
