out(
	#sine(audiofile("/home/merayen/ダウンロード/splash-6213.mp3")*440 + 440)
	mix(sine(700), sine(880), square(1)) * square(2)
)
