"""
Importing data to aim.

Usually converts files to our own format and store it in .data for performant use.
"""
import hashlib
import os
import subprocess

def get_ffprobe_file_information(path: str) -> list[dict[str, str]]:
	process = subprocess.Popen(
		["ffprobe", "-hide_banner", "-show_streams", path],
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
	)
	stdout, stderr = process.communicate()

	assert not process.returncode, f"ffprobe returned code {process.returncode}:\n{stderr.decode('utf-8')}"

	streams = []
	for line in stdout.decode("utf").splitlines():
		line = line.strip()
		if line == "[STREAM]":
			streams.append({})
			continue

		if line == "[/STREAM]":
			continue

		k,v = line.split("=", 1)
		streams[-1][k] = v

	return streams


	# XXX Note: This mixes all streams. Can be buggy if e.g multiple audio streams or video streams
	# overlap with audio stream
	return dict(x.split("=", 1) for x in stdout.decode("utf-8").splitlines() if "=" in x)


def read_file_data(path: str) -> tuple[str, list[str]]:
	"""
	Read audio from a file and store it into local .data folder for easy access

	Use "aim reload" to reload all source files, or just delete .data folder yourself.
	"""
	if not os.path.isdir(".data"):
		# Temporary data folder that is user local. Can be deleted to save space.
		os.mkdir(".data")

	# Make a hash of the path for simplicity
	file_basename = hashlib.md5(path.encode("utf")).hexdigest().lower()

	# Return any existing files with this hash
	return (
		file_basename,
		[
			".data" + os.path.sep + filename
			for filename in os.listdir(".data")
			if filename.startswith(file_basename + "-")
		],
	)


def read_audio_data(path: str, sample_rate: int) -> list[str]:
	"""
	Return all the paths for each audio channel retrieved

	Audio is encoded into float32 and each file represents a single channel.

	If file has already been cached, we don't attempt to reload it.

	User needs to either delete ".data" folder or run "aim reload" to retrieve any
	new changes in the source files. Especially important when changing sample rate
	of the project.
	"""
	hashsum, existing_files = read_file_data(path)
	if existing_files:
		# There exists already cached files for this path. Just use that one without any more
		# verification. User also needs to purge the .data directory to have aim to re-read.
		# This makes aim more performant playing a project.
		return existing_files

	# Not cached yet. Load it
	ffprobe_info = get_ffprobe_file_information(path)

	audio_streams = [x for x in ffprobe_info if x.get("codec_type") == "audio"]

	assert audio_streams, f"No audio streams in file '{path}'"

	assert len(audio_streams) == 1, f"Multiple audio streams found in file '{path}'. This is not yet supported"

	channels = int(audio_streams[0]["channels"])

	assert channels > 0, f"No audio channels found in file {path}"

	# Create arguments for ffmpeg to split the audio channels into separate files
	ffmpeg_audio_mappings = []
	for i in range(channels):
		ffmpeg_audio_mappings.extend(
			[
				f"-map_channel",
				# XXX this might get wrong channel mapping when just iterating...?
				f"0.{audio_streams[0]['index']}.{i}",
				"-f", "f32le",
				"-ar", str(sample_rate),
				f".data/{hashsum}-{i}",
			]
		)


	# Ask ffmpeg to convert the file
	process = subprocess.Popen(
		[
			"ffmpeg",
			"-hide_banner",
			"-i", path,
			*ffmpeg_audio_mappings,
		],
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
	)
	stdout, stderr = process.communicate()

	assert not process.returncode, f"ffmpeg returned code {process.returncode}:\n{stderr.decode('utf-8')}"

	return read_file_data(path)[1]
