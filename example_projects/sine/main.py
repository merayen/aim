out(
	oscilloscope(sine(sine(4) * 110 + 440) + square(sine(2) * 220 + 440), time_div=1E-2)
)
