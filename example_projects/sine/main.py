_a=sine(.5)

out(
	oscilloscope(
		trigger(
			sine(220*2.1),
			start=_a,
			stop=_a,
		) * square(10)
	)
)
