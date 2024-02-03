_a=sine(.5)

out(
	oscilloscope(
		sine(sine(.1) * 200 + 210),
		trigger=0.9,
		time_div=0.01,
	)
)
