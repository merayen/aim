from math import sin
from numba import njit, prange, jit

slew_rate = .1
ringing = 0
samplerate = 48000



@njit(nogil=True, cache=True, nopython=True)
def calculate(slew_rate: float, ringing: float, samplerate: int):
	m = 0
	o = 0

	r = []
	s = []
	for i in prange(samplerate*5):
		v = int((i/samplerate)%1>=.5)*2 - 1
		m += (v - o - m) / (1 + ringing * samplerate)
		o += m / (1 + slew_rate * samplerate)
		r.append(o)
		s.append(v)

	return r, s


# Warm up
import time
_t = time.time()
calculate(slew_rate, ringing, samplerate)
print(time.time()-_t)

_t = time.time()
for i in range(100):
	r, s = calculate(slew_rate, ringing, samplerate)
print(time.time()-_t)
#calculate.parallel_diagnostics(level=4)


# Numpy implementation
t = []
...

# Graphs

import pylab as pl
fig, axs = pl.subplots(1)
axs.plot(s, label="input")
axs.plot(r, label="output")
axs.legend()
pl.get_current_fig_manager().window.showMaximized()
pl.waitforbuttonpress()
pl.close()
