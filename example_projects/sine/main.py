out(
	#sine(audiofile("/home/merayen/ダウンロード/splash-6213.mp3")*440 + 440)
	trigger(sine(440), start=0.01, stop=-0.01)* 2 - 1
)
